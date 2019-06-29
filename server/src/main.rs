use std::io::{ErrorKind, Read, Write}; //ErrorKind: error message type, 
use std::net::TcpListener; //allow us to create our server and listen on a port
use std::sync::mpsc; //allow us to spawn a channel
use std::thread; //allow us to work with multiple threads

const LOCAL: &str = "127.0.0.1:6000"; //localhost with port in it
const MSG_SIZE: usize = 64;//was 32 //the buffer size of our messages, want it at most 32-bit in size

//Will allow our thread to sleep for a moment, while its not receiving messages
fn sleep_thread() {
    thread::sleep(::std::time::Duration::from_millis(100)); //100 mili seconds
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind"); //instantiate server
    server.set_nonblocking(true).expect("failed to initialize non-blocking"); //let server constantly check for messages

    let mut clients = vec![]; //users
    let (tx,rx) = mpsc::channel::<String>(); //instantiate channel and say we gonna send Strings into it (transmitter, receiver) 
    loop {
        if let Ok((mut socket, addr)) = server.accept() { //expect connection, it means it worked 
            println!("Client {} connected", addr);

            //Cloning so we can use them inside our threads
            let tx = tx.clone(); //transmitter
            clients.push(socket.try_clone().expect("failed to clone client")); //add user to clients

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE]; //full of 0s with message size

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        //take msg we receive -> convert to iterator then take all characters that r not white-space and collect them in vector
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        
                        //change that slice of string into actual string
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        let sender_msg = format!("{}: {:?}", addr, msg);
                        println!("{}", sender_msg); //println!("{}: {:?}", addr, msg);

                        //Send message to receiver
                        tx.send(sender_msg).expect("failed to send msg to rx"); //was msg
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (), // its error that would block, send unit type so we can continue
                    Err(_) => { //Error that we don't care what inside of it, then close the loop
                        println!("closing connection with: {}", addr);
                        break;
                    }
                }
                //rest between each loop
                sleep_thread();
            });
        }

        //When server receives a message and we send it by transmitter inside above loop
        if let Ok(msg) = rx.try_recv() {
            //convert clients to interator and filter on it, then collect it all into vector
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes(); //convert messages to bytes
                buff.resize(MSG_SIZE, 0);   //resize buffer based on message size
            
                client.write_all(&buff).map(|_| client).ok() //write all entire buffer & map it into our client & send it back 
            }).collect::<Vec<_>>();
        }

        sleep_thread();
    }
}
