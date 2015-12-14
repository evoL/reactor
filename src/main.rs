extern crate rand;
extern crate byteorder;
extern crate num_cpus;
extern crate crossbeam;

use types::{System, Particle};
use types::transform::*;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc;

const ADDRESS: &'static str = "127.0.0.1:24267";
const MAX_TTL: i32 = 30;
const PARTICLE_COUNT: i32 = 10000;
const ITERATION_COUNT: i32 = 1000;

enum Message {
    Generated(Particle),
    Finished,
}

fn handle_client(mut stream: TcpStream) {
    let mut global_rng = rand::thread_rng();
    let system = System { ttl: MAX_TTL };

    let variation = variations::DeJong(-1.860391774909643026, 1.100373086160729041, -1.086431197851741803, -1.426991546514589704);
    let transform = TransformBuilder::new()
        .add_variation(variation)
        .color(1.0)
        .finalize();
    let final_transform = TransformBuilder::new()
        .add_variation(variations::Linear)
        .color(1.0)
        .finalize();

    let thread_count = num_cpus::get();
    let chunk_size = ((PARTICLE_COUNT as f32) / (thread_count as f32)).ceil() as usize;
    let mut particles: Vec<Particle> = (0..PARTICLE_COUNT).map(|_| system.make_particle(&mut global_rng)).collect();

    crossbeam::scope(|scope| {
        let (tx, rx) = mpsc::channel();

        for particle_chunk in particles.chunks_mut(chunk_size) {
            let (tx, system, transform, final_transform) = (tx.clone(), &system, &transform, &final_transform);

            scope.spawn(move|| {
                let mut rng = rand::thread_rng();

                for _ in 0..ITERATION_COUNT {
                    for particle in particle_chunk.iter_mut() {
                        system.animate_particle_mut(particle, transform, &mut rng);
                        let projected_particle = final_transform.animate(&particle);

                        tx.send(Message::Generated(projected_particle)).unwrap();
                    }
                }

                tx.send(Message::Finished).unwrap();
            });
        }

        let mut finished_threads: usize = 0;
        while finished_threads < thread_count {
            let message = rx.recv().unwrap();

            match message {
                Message::Generated(particle) => {
                    let _ = stream.write(&particle.bytes());
                },
                Message::Finished => finished_threads += 1
            }
        }
    });

    println!("Payload sent");
}

fn main() {
    let listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Reactor is listening on {}", ADDRESS);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Sending payload…");
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}

mod types;
mod consts;
mod variations;
