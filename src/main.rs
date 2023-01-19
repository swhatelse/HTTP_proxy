extern crate clap;
use clap::Parser;
use tokio::{net::{TcpListener, TcpStream}};
use tokio_util::codec::{BytesCodec, FramedRead};
use httparse::{Request};
use futures_util::stream::StreamExt;
use std::option::Option;
// use bytes::BytesMut;

#[derive(Parser, Debug)]
struct Options {
    destination: String,
}

fn request_to_str(req: Request) -> String{
    let mut s = "GET / HTTP/1.1\r\n".to_string();
    
    for h in req.headers {
        s = s + h.name + ": " + &String::from_utf8_lossy(h.value) + "\r\n";
    }

    s = s + "\r\n";
    s        
}

#[derive(Copy, Clone)]
struct Proxy<'a> {
    dst: &'a str,
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
        let mut dst = TcpStream::connect(self.dst).await?;
        
        let (mut dst_r, mut dst_w) = dst.split();

        // Modify header request
        let mut buf = "".to_string();
        let res = self.parse_http_request(&mut src).await;        
        match res {
            Ok(Some(res)) => buf = res,
            Ok(None) => todo!(),
            Err(_e) => println!("Error while parsing"), // TODO handle error properly
        }
        
        // let req = res.unwrap().unwrap();
        let mut buf = buf.as_bytes();
        // println!("{:?}", String::from_utf8_lossy(buf));
        
        let handle1 = async {
            tokio::io::copy(&mut buf, &mut dst_w).await
        };

        // TODO modify the response
        let (_src_r, mut src_w) = src.into_split();
        let handle2 = async {
            tokio::io::copy(&mut dst_r, &mut src_w).await
        };

        tokio::try_join!(handle1, handle2)?;
        
        Ok(())
    }

    async fn parse_http_request<'b>(&self, stream: &mut TcpStream) -> Result<Option<String>,  &'static str>{
        let mut framed = FramedRead::new(stream, BytesCodec::new());
        let mut modified_hdr = "".to_string();

        // Read incoming data from the client
        match framed.next().await.unwrap(){
            Ok(bytes) => {
                let mut headers = [httparse::EMPTY_HEADER; 16];
                let mut req = Request::new(&mut headers);
                // println!("{:?}", String::from_utf8_lossy(bytes.as_ref()));
                let res = req.parse(bytes.as_ref());    
                
                match res {
                    Ok(_) => {
                        req.headers[0].value = self.dst.as_bytes(); // Change the address destination
                        modified_hdr = request_to_str(req);         // Regenerate modified headers
                    }
                    Err(_e) => return Err("Failed to parse")
                }
            }
            Err(_e) => return Err("Failed to read the buffer"),
        }

        Ok(Some(modified_hdr))
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
