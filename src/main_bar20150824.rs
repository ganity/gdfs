use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
//use std::sync::mpsc;
//use std::sync::mpsc::Sender;
//use std::sync::mpsc::Receiver;
use std::error::Error;
use std::io::prelude::*;

use std::io::SeekFrom;
use std::fs::OpenOptions;

extern crate tiny_http;
use tiny_http::{ServerBuilder, Response, Header};

fn main(){
	//sync
	//let data = Arc::new(Mutex::new(0));
	//make channel
	//let (tx, rx) = mpsc::channel();

	let listener = TcpListener::bind("10.16.21.162:8088").unwrap();
	fn u8toi32(src: [u8; 4]) -> i32{
		( (((src[0] as i32) & 0xFF)<<24)  
        |(((src[1] as i32) & 0xFF)<<16)  
        |(((src[2] as i32) & 0xFF)<<8)  
        |((src[3] as i32) & 0xFF)) as i32 
    	//value = (src[0] as i32) * 256 * 256 * 256 + (src[1] as i32) * 256 * 256 + (src[2] as i32) * 256 + (src[3] as i32); 
       // println!("value={}", value);
	}
	fn i32tou8(value: i32) -> [u8; 4]{
		//let value: i32 = 45;
		let mut src = [0; 4];  
	    src[0] = ((value>>24) & 0xFF) as u8;  
	    src[1] = ((value>>16)& 0xFF) as u8;  
	    src[2] = ((value>>8)&0xFF) as u8;    
	    src[3] = (value & 0xFF) as u8; 
	    //u8toi32(src);
	    src
	}
	
	//start a dispatcher server
	fn dispatcher(len: i32) -> (String, i32) {
		// thread::spawn(move || {
		 	let data = Arc::new(Mutex::new(0));
		 	let mut index = 4;
		// 	let (tx, rx) = channel();
		// 	tx.send(21).unwrap();
		 	//tx.send(21).unwrap();
		 	let data = data.clone();
		 	//loop {
		// 		//let (data, tx) = (data.clone(), tx.clone());
		 		//let rc = rx.recv().unwrap();
		 		let _ = data.lock().unwrap();
		 		let mut f = OpenOptions::new().read(true).write(true).open("data.dat").unwrap();
		 		//*data += 1;
		 		//tx.send("rc".to_string()).unwrap();
				let mut head = [0; 4];
				let _ = f.read(&mut head);
				let lenth = u8toi32(head);
				if lenth != 0 {
					index = lenth;
				}
				
		 		//println!("head{}", lenth);

		 		
		 		//println!("{}", rc);
		 		//println!("{}", len);
		 		//index += len;
				// let a = u8toi32(i32tou8(index));
				 //println!("index={}", index);
				let u84 = i32tou8(index + len);

				let _ =  f.seek(SeekFrom::Start(0));
		 		let _ =  f.write_all(&u84).unwrap();

		 		
		 		("fileid/".to_string() + &index.to_string() + "/" + &len.to_string() + "/aaa", index)
		 	//}
		 //});
	}

	fn filereader(mut stream: TcpStream){
		let mut f = OpenOptions::new().read(true).open("data.dat").unwrap();
		
		let mut fid = [0u8; 1024];
		//let mut s = "";
		match stream.read(&mut fid){
			Err(why) => panic!("couldn't read {}",
						   Error::description(&why)),
			Ok(n) => {
				// s = &fid[0..n];
				// println!("file s={}", s);
				let s = String::from_utf8_lossy(&fid[0..n]);
			    //println!("file s={}", s);
			    let v: Vec<&str> = s.split('/').collect();
			    let index = v[1].parse::<i32>().unwrap();
			    let mut length = v[2].parse::<usize>().unwrap();
			    length += 1024;
			    let _ = f.seek(SeekFrom::Start(index as u64));
				while length > 1024 {
					//println!("file length:{}", n as i32);
					let mut l = [0u8; 1024];
					let mut wrlen = 1024;
					if (length -1024) < 1024 {
						wrlen = length - 1024;
					} 
					match f.read(&mut l){
						Err(why) => panic!("couldn't read {}",
									   Error::description(&why)),
						Ok(_) =>{
							//length = length +n;
							//println!("file contains:{}", n as i32);
							stream.write_all(&l[0..wrlen]).unwrap();
						},
					}

					length -= 1024;
				}
			},
		}
	
	}

	fn filewrite(mut stream: TcpStream, start :i32, fid: String, mut len: i32){
	//fn filewrite(mut stream: TcpStream, buf: &[u8]){
		 // let mut f = File::create("data.dat").unwrap();
		 //let mut f = File::open("data.dat").unwrap();
		 let mut f = OpenOptions::new().read(true).write(true).open("data.dat").unwrap();
		 //println!("start={}", start);
		let _ = f.seek(SeekFrom::Start(start as u64));

		while len > 0 {
			let mut l = [0u8; 1024];
			match stream.read(&mut l){
				Err(why) => panic!("couldn't read {}",
							   Error::description(&why)),
				Ok(n) =>{
					//println!("contains:{}", n as i32);
					f.write_all(&l[0..n]).unwrap();
				},
			}

			len -= 1024;
		}
		let _ = stream.write(fid.as_bytes());
	}

	//accepter accept connector 
	//1.recive file key and response the file data, 
	//2.update file return the file key  
	fn handle_client(mut stream: TcpStream) {
		//let mut s = String::new();
		//stream.read_to_string(&mut s);
		// let mut l0 = [0;1];
		// let mut l1 = [0;1];
		// let mut l2 = [0;1];
		// let mut l3 = [0;1];
		// stream.read(&mut l0);
		// stream.read(&mut l0);
		// stream.read(&mut l0);
		// stream.read(&mut l0);
		// let it = &mut [l0[0], l1[0], l2[0], l3[0]];

		// //read buf length
		let mut l = [0u8; 4];
		let _ = stream.read(&mut l);
		let lenth = u8toi32(l);
		// //read request type 0=update file, 1=get file
		let mut t = [0u8; 1];
		let _ = stream.read(&mut t);

		if t[0] == 0 {
			// for x in l.iter() {
			// 	println!("{}", x);
			// }
			
			//(l[0] as i32) * 256 * 256 * 256 + (l[1] as i32) * 256 * 256 + (l[2] as i32) * 256 + (l[3] as i32);
			 //println!("{}", lenth);
			
			// 	//Err(why) => panic!("couldn't read {}",
			// 					 //  Error::description(&why)),
			// 	//Ok(_) =>
			// let fid = dispatcher(lenth);
			// println!("{}", fid);
			// let len = lenth as usize;
			// let mut file = [0u8;len];
			// stream.read(&mut file);
			// let mut buffer = Vec::new();
			// stream.read_to_end(&mut buffer);
			let (fid, start) = dispatcher(lenth);
			
			//println!("{}", got);
			//buf.slice(0, got)
			//println!("{}", l.len());
			filewrite(stream, start, fid, lenth);
		} else {
			filereader(stream);
		}
	
	}

	//http server
	thread::spawn(move || {
		let server = ServerBuilder::new().with_port(8000).build().unwrap();

		for request in server.incoming_requests() {
		   
			thread::spawn(move || {
			    let mut f = OpenOptions::new().read(true).open("data.dat").unwrap();
			    let response = {
				    let fid = request.url().trim_left_matches("/");
				    if fid.starts_with("fileid"){
				    
					    let v: Vec<&str> = fid.split('/').collect();
						let index = v[1].parse::<i32>().unwrap();
						let length = v[2].parse::<usize>().unwrap();

					    let mut rps = vec![0; length];
						let _ = f.seek(SeekFrom::Start(index as u64));
						match f.read(&mut rps){
							Err(why) => panic!("couldn't read {}",
										   Error::description(&why)),
							Ok(_) =>{
							},
						}
						let cthv: Vec<&str> = v[3].rsplit('.').collect();
						
						let content_type_header = {
							match cthv[0] {
							    "jpg" => "Content-Type: image/jpg",
							    "txt" => "Content-Type: text/plain",
							    "png" =>"Content-Type: image/png",
							    "html"|"htm"|"xhtml" =>"Content-Type: text/html",
							    "xml"|"tsd"|"xsd" =>"Content-Type: text/xml",
							    _ => "Content-Type: application/octet-stream",
							}
						}.parse::<Header>().unwrap();
					    //let content_type_header = "Content-Type: image/jpg".parse::<Header>().unwrap();
					    let cache_control_header = "cache-control: no-cache".parse::<Header>().unwrap();
					     
					     Response::from_data(rps).with_header(content_type_header).with_header(cache_control_header)
				    } else {
				     	Response::from_string("hello world")
				    }
				};
			    // let response = Response::from_string("hello world");
			    request.respond(response);
			});
		}
	});

	// accept connections and process them, spawning a new thread for each one
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				thread::spawn(move || {
					// connection succeeded
					handle_client(stream)
				});
			}
			Err(e) => { /* connection failed */
				panic!("connection failed {}",  Error::description(&e))
			}
		}
	}

	// close the socket server
	drop(listener);
}