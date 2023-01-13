extern crate clap;
use clap::Parser;
use tokio::{net::{TcpListener, TcpStream}};

#[derive(Parser, Debug)]
struct Options {
    destination: String,
}

#[derive(Copy, Clone)]
struct Proxy<'a> {
    dst: &'a str,
}

impl<'a> Proxy<'_> {

    pub fn new(dest: &'a str) -> Proxy<'a>{
        Proxy{ dst: dest }
    }

    pub async fn start(&'a self)  -> std::io::Result<()>{
        let listener = TcpListener::bind("127.0.0.1:15411").await?;

        loop {
            let (connection, _) = listener.accept().await?;
                self.handle_connection(connection).await?;
        }
    }
    
    pub async fn handle_connection(&'a self, mut src: TcpStream) -> std::io::Result<()>{
        let mut dst = TcpStream::connect(self.dst).await?;
        
        let (mut src_r, mut src_w) = src.split();
        let (mut dst_r, mut dst_w) = dst.split();


        let handle1 = async {
            tokio::io::copy(&mut src_r, &mut dst_w).await
        };
        
        let handle2 = async {
            tokio::io::copy(&mut dst_r, &mut src_w).await
        };

        tokio::try_join!(handle1, handle2)?;
        
        Ok(())
    }

}

#[tokio::main]
async fn main() -> std::io::Result<()>{
    let opts = Options::parse();

    let dst_addr = opts.destination.as_str();
    let proxy = Proxy::new(dst_addr);
    proxy.start().await?;
    
    Ok(())
}
