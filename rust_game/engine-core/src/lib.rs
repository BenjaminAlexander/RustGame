mod messaging;
mod server;
//TODO: maybe move the interface out of a module and place it directly under the crate
pub mod interface;
mod gametime;
//TODO: pull public pieces from messaging into the interface
pub mod client;
mod gamemanager;