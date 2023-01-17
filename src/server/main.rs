extern crate clap;
use clap::Parser;
use tokio::{net::{TcpListener, TcpStream}, io::{BufReader, AsyncBufReadExt}};
use tokio_util::codec::{BytesCodec, FramedRead};
use httparse::{Request, Status};
use futures_util::stream::StreamExt;
// use bytes::BytesMut;

#[derive(Parser, Debug)]
struct Options {
    destination: String,
}

#[derive(Copy, Clone)]
struct Proxy<'a> {
    dst: &'a str,
}

/**
 * Parse a HTTP headers
 */
fn parse<'b>(buf: &'b[u8], req: &'b mut Request<'b, 'b>){
    let res = req.parse(buf);    

    match res {
        Ok(_) => {
            println!("{:?}",  String::from_utf8_lossy(req.headers[0].value));
        }
        Err(e) => println!("Error: {:?}", e)
    }
}

async fn read_http_request<'b>(stream: &mut TcpStream) -> std::io::Result<()>{
    let mut framed = FramedRead::new(stream, BytesCodec::new());
    
    while let Some(bytes) = framed.next().await{
        let buf = bytes.unwrap();
        let buf = buf.as_ref();
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = Request::new(&mut headers);
        parse(buf, &mut req);
    }
    
    Ok(())
}

impl<'a> Proxy<'_> {

    pub fn new(dest: &str) -> Proxy{
        Proxy{ dst: dest }
    }

    pub async fn start(&self)  -> std::io::Result<()>{
        let listener = TcpListener::bind("127.0.0.1:15411").await?;
        
        loop {
            let (connection, _) = listener.accept().await?;
            self.handle_connection(connection).await?;
        }
    }
    
    pub async fn handle_connection(&self, mut src: TcpStream) -> std::io::Result<()>{
        // let mut dst = TcpStream::connect(self.dst).await?;
        
        // let (mut src_r, mut src_w) = src.into_split();
        // let (mut dst_r, mut dst_w) = dst.split();

        // TODO: modify header request
        tokio::try_join!(read_http_request(&mut src))?;
        
        // let handle1 = async {
        //     tokio::io::copy(&mut src_r, &mut dst_w).await
        // };
        
        // let handle2 = async {
        //     tokio::io::copy(&mut dst_r, &mut src_w).await
        // };

        // tokio::try_join!(handle1, handle2)?;
        
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

#[cfg(test)]
mod test {

}
