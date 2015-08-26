use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
//use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
//use std::sync::mpsc::Receiver;
use std::error::Error;

use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::io::SeekFrom;
use std::fs::OpenOptions;

use std::hash::{Hash, SipHasher, Hasher};

extern crate tiny_http;
use tiny_http::{ServerBuilder, Response, Header};



//hash the file
#[derive(Hash)]
struct HashFile {
    start: i32,
    length: i32,
}
fn file_hash<T>(obj: T) -> u64 where T: Hash {
    let mut hasher = SipHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

fn main(){
	

	let listener = TcpListener::bind("10.16.21.162:8088").unwrap();
	fn u8toi32(src: [u8; 4]) -> i32{
		( (((src[0] as i32) & 0xFF)<<24)  
        |(((src[1] as i32) & 0xFF)<<16)  
        |(((src[2] as i32) & 0xFF)<<8)  
        |((src[3] as i32) & 0xFF)) as i32 
    	
	}
	fn i32tou8(value: i32) -> [u8; 4]{
		
		let mut src = [0; 4];  
	    src[0] = ((value>>24) & 0xFF) as u8;  
	    src[1] = ((value>>16)& 0xFF) as u8;  
	    src[2] = ((value>>8)&0xFF) as u8;    
	    src[3] = (value & 0xFF) as u8; 
	    src
	}
	
	fn check_hash(index: i32, length: i32, chev: &str) -> bool {
		let hf = HashFile { start: index, length: length as i32 };
 		let av: Vec<&str> = chev.split('.').collect();
 		match av[0].parse::<u64>() {
 			Err(_) =>false,

			Ok(n) => n == file_hash(hf),
		}
	}

	//sync dispatcher
	let data = Arc::new(Mutex::new(0));
	//make channel
	let (tx, rx) = mpsc::channel();
	//start a dispatcher server
	//fn dispatcher(len: i32, tx: mpsc::Sender<(Vec<usize>) -> (String, i32) {
		 thread::spawn(move || {
		 
		 	loop {
		
		 		let _ = data.lock().unwrap();
		 		let (txx, len): (Sender<(String, i32)>, i32) = rx.recv().unwrap();
		 		
		 		
		 		let mut f = OpenOptions::new().read(true).write(true).open("data.dat").unwrap();

		 	
				let mut head = [0; 4];
				let _ = f.read(&mut head);
				let index = u8toi32(head);
				
				let u84 = i32tou8(index + 1);

				let _ =  f.seek(SeekFrom::Start(0));
		 		let _ =  f.write_all(&u84).unwrap();

		 		let hf = HashFile { start: index, length: len };
		 		txx.send(("fileid/".to_string() + &index.to_string() + "/" + &len.to_string() + "/" + &file_hash(hf).to_string(), index)).unwrap();
		 		
		 	}
		 });
	//}

	fn filereader(mut stream: TcpStream){
		
		
		let mut fid = [0u8; 1024];
		
		match stream.read(&mut fid){
			Err(why) => panic!("couldn't read {}",
						   Error::description(&why)),
			Ok(n) => {
				let s = String::from_utf8_lossy(&fid[0..n]);
			    let v: Vec<&str> = s.split('/').collect();
			    let index = v[1].parse::<i32>().unwrap();
			    let mut length = v[2].parse::<usize>().unwrap();

		 		if check_hash(index, length as i32, v[3]) {

				    
				    let fp = "data/".to_string() + &(index%64).to_string()+"/"+v[3];
				    let mut f = OpenOptions::new().read(true).open(fp).unwrap();
				   // let _ = f.seek(SeekFrom::Start(index as u64));
					let mut l = vec![0; length];
					while length > 0 {
						match f.read(&mut l){
							Err(why) => panic!("couldn't read {}",
										Error::description(&why)),
							Ok(n) =>{
							
								if n > length {
									stream.write_all(&l[0..length]).unwrap();
								} else {
									stream.write_all(&l[0..n]).unwrap();
								}
									length -= n;
							},
						}
					}
				} else {
					stream.write_all(b"Error file id check_hash!").unwrap();
				}
			},
		}
	
	}

	
	//start a file writer
	// let (ftx, frx) = mpsc::channel();
	// let fdata = Arc::new(Mutex::new(0));
	// thread::spawn(move || {
	// 	loop {
	 		
	//  		let (start, buf, len): (i32, Vec<u8>, usize) = frx.recv().unwrap();
	//  		let _ = fdata.lock().unwrap();
	//  		//let len = rx1.recv().unwrap();
	//  		let mut f = OpenOptions::new().read(true).write(true).open("data.dat").unwrap();
	//  		let _ = f.seek(SeekFrom::Start(start as u64)).unwrap();
	//  		println!("start={}, len={}", start, len);
			
	// 		f.write_all(&buf[0..len]).unwrap();
	//  	}
	//  });

	for i in 0..64 {
		let dir = "data/".to_string()+&i.to_string();
		let _ = fs::create_dir(&dir);
	}
	

	fn filewrite(mut stream: TcpStream, fid: String, mut len: i32){
	
		let v: Vec<&str> = fid.split('/').collect();
		let st = v[1].parse::<usize>().unwrap();
		//println!("fid={}, dir={}", fid, st%64);
		let a = "data/".to_string()+"/"+&(st%64).to_string()+"/"+v[3];

		let mut f = File::create(&a).unwrap();
		while len > 0 {
			let mut l = vec![];
			match stream.read_to_end(&mut l){
				Err(why) => panic!("couldn't read {}",
							Error::description(&why)),
				Ok(n) =>{
					//println!("contains:{}", n as i32);
					len -= n as i32;
					f.write_all(&l[0..n]).unwrap();
					//start += n as i32;
					//ftx.send((start, l, n)).unwrap();
				},
			}

		}

		let _ = stream.write(fid.as_bytes());
	}
	// });

	//accepter accept connector 
	//1.recive file key and response the file data, 
	//2.update file return the file key  
	fn handle_client(mut stream: TcpStream, tx: Sender<((Sender<(String, i32)>, i32))>) { //, ftx: Sender<(i32, Vec<u8>, usize)>
		
		
		// //read buf length
		let mut l = [0u8; 4];
		let _ = stream.read(&mut l);
		let lenth = u8toi32(l);
		//read request type 0=update file, 1=get file
		let mut t = [0u8; 1];
		let _ = stream.read(&mut t);
		
		if t[0] == 0 {
			let (tx1, rx1) = mpsc::channel();
			tx.send((tx1, lenth)).unwrap();

			let (fid, _) = rx1.recv().unwrap();//("ffff", 4);//dispatcher(lenth);

			filewrite(stream, fid, lenth);
			
		} else {
			filereader(stream);
		}
	
	}

	//http server
	thread::spawn(move || {
		let server = ServerBuilder::new().with_port(8000).build().unwrap();

		for request in server.incoming_requests() {
		   
			thread::spawn(move || {
			    // let mut f = OpenOptions::new().read(true).open("data.dat").unwrap();
			    let response = {
				    let fid = request.url().trim_left_matches("/");
				    if fid.starts_with("fileid"){
				    
					    let v: Vec<&str> = fid.split('/').collect();
						let index = v[1].parse::<i32>().unwrap();
						let length = v[2].parse::<usize>().unwrap();

						if check_hash(index, length as i32, v[3]) {
							let cthv: Vec<&str> = v[3].split('.').collect();
							let fp = "data/".to_string() + &(index%64).to_string()+"/"+cthv[0];
				    		let mut f = OpenOptions::new().read(true).open(fp).unwrap();
						    let mut rps = vec![0; length];
							// let _ = f.seek(SeekFrom::Start(index as u64));
							match f.read(&mut rps){
								Err(why) => panic!("couldn't read {}",
											   Error::description(&why)),
								Ok(_) =>{},
							}
							
							
							let content_type_header = {
								if cthv.len() == 2 {
									match cthv[1] {
									    "jpg" => "Content-Type: image/jpg",
									    "txt" => "Content-Type: text/plain",
									    "png" =>"Content-Type: image/png",
									    "html"|"htm"|"xhtml" =>"Content-Type: text/html",
									    "xml"|"tsd"|"xsd" =>"Content-Type: text/xml",
									    _ => "Content-Type: application/octet-stream",
									}
								} else {
									"Content-Type: application/octet-stream"
								}
							}.parse::<Header>().unwrap();
						    //let content_type_header = "Content-Type: image/jpg".parse::<Header>().unwrap();
						    let cache_control_header = "cache-control: no-cache".parse::<Header>().unwrap();
						     
						    Response::from_data(rps).with_header(content_type_header).with_header(cache_control_header)
						} else {
							Response::from_string("Error file id check_hash!")
						}
				    } else {
				     	Response::from_string("Error file id!")
				    }
				};
			    // let response = Response::from_string("hello world");
			    request.respond(response);
			});
		}
	});

	
	// accept connections and process them, spawning a new thread for each one
	for stream in listener.incoming() {
		let tx = tx.clone();
		match stream {
			Ok(stream) => {
				thread::spawn(move || {
					// connection succeeded
					handle_client(stream, tx)
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