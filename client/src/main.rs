use std::io::{self, ErrorKind, Read, Write}; //self: want to import io library itself, ErrorKind: error message type
use std::net::TcpStream; //we have TcpListener in server app
use std::sync::mpsc::{self, TryRecvError}; //A channels library
use std::thread;   //allow us to use threads
//use std::time::Duration; 

extern crate colored; //customiz color of our println
use colored::*;

//clear commandline lines 
use console::Term; 

extern crate notifica; //send notificaiton when receive message
use notifica::*;

const LOCAL: &str = "127.0.0.1:6000"; //localhost with port in it
const MSG_SIZE: usize = 64;//was 32 //the buffer size of our messages, want it at most 32-bit in size


fn main() {
    //Instantite our client 
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non blocking"); //let server constantly check for messages

    //Instantiate our channel (transmitter and receiver), send string through it again 
    let (tx, rx) = mpsc::channel::<String>();

    //Mahmoud - For Clearing what we typed on commandline
    let term = Term::stdout();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE]; //buffer vector of 0s inside of it is size message  
    
        //MARK: User Receive Messages
        //Read (Our and other users) messages through the buffer
        match client.read_exact(&mut buff) {
            Ok(_) => {
                //buffer turned into iterator, check if refrences inside of it are equal to 0, then collect them inside Vector   
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();

                //change that slice of string into actual string
                let msg_str = String::from_utf8(msg.clone()).expect("Invalid utf8 message");
                
                
                println!("\n~~~ Message recevied ~~~");//println!("\n~~~ Message recevied ~~~\n{}\n{:?}\n", msg_str, msg);
                println!("{}", msg_str.green().bold());
                println!("{:?}\n", msg);
                //notifica::notify("Message Recevied", "ajkshfjklhadfs"); gives error right now
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (), // its error that would block, send unit type so we can continue
            Err(_) => { //Get another type of error, break out of loop
                println!("{}" , "Debug- Connection with server was severed".cyan());
                break;
            }
        }    

        //MARK: User Send Messages
        //Check if server sends back a message, that says it got the message that we were sending from client 
        match rx.try_recv() {
            Ok(msg) => { //if got the message
                //clone message into bytes and put it inside puffer
                let mut buff = msg.clone().into_bytes();
                
                //resize buffer by message size
                buff.resize(MSG_SIZE, 0);

                //write all our buffers into our client
                client.write_all(&buff).expect("Debug- Writing to socket failed");

                println!("{} {:?}","Debug- We sent message:".cyan(), msg);
            },
            Err(TryRecvError::Empty) => (), //send back unit type
            Err(TryRecvError::Disconnected) => break //disconnected type, break out of loop
        }

        //rest between each loop
        sleep_thread();
    });

    //App Start
    println!("\n\n{}","Write a Message: ".bold().purple());
    //MARK: User Send Messages
    loop {
        //create empty string
        let mut buff = String::new(); 
        //read the command line and add it to our string
        io::stdin().read_line(&mut buff).expect("Debug- reading from stdin failed");
        term.clear_last_lines(1).expect("Debug- Couldn't delete that last 1 line");
        println!();

        //trim it and make it into string
        let msg = buff.trim().to_string();

        //if there was error white sending message or we typed ":quit" then stop and break out of loop
        if msg == ":quit" || tx.send(msg).is_err() { break }
    }
    println!("{}", "Bye bbye!".red());
}

//Will allow our thread to sleep for a moment, while its not receiving messages
fn sleep_thread() {
    thread::sleep(::std::time::Duration::from_millis(100)); //100 mili seconds
}