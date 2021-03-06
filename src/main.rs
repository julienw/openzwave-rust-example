extern crate openzwave;
use openzwave::{options, manager, notification, controller};
use openzwave::notification::*;
use openzwave::node::*;
use std::{fs, io};
use std::sync::Mutex;
use std::collections::{ HashMap, HashSet };
use std::io::Write;

#[cfg(windows)]
fn get_default_device() {
    "\\\\.\\COM6"
}

#[cfg(unix)]
fn get_default_device() -> Option<&'static str> {
    let default_devices = [
        "/dev/cu.usbserial", // MacOS X
        "/dev/cu.SLAB_USBtoUART", // MacOS X
        "/dev/ttyUSB0" // Linux
    ];

    default_devices
        .iter()
        .find(|device_name| fs::metadata(device_name).is_ok())
        .map(|&str| str)
}

struct ProgramState {
    controllers: HashSet<controller::Controller>,
    nodes: HashSet<Node>,
    nodes_map: HashMap<controller::Controller, Vec<Node>>,
}

impl ProgramState {
    fn new() -> ProgramState {
        ProgramState {
            controllers: HashSet::new(),
            nodes: HashSet::new(),
            nodes_map: HashMap::new()
        }
    }
}

struct Program {
    state: Mutex<ProgramState>
}

impl Program {
    pub fn new() -> Program {
        Program {
            state: Mutex::new(ProgramState::new())
        }
    }
}

impl manager::NotificationWatcher for Program {
    fn on_notification(&self, notification: Notification) {
        //println!("Received notification: {:?}", notification);

        match notification.get_type() {
            NotificationType::Type_DriverReady => {
                let controller = notification.get_controller();
                let mut state = self.state.lock().unwrap();
                if !state.controllers.contains(&controller) {
                    println!("Found new controller: {:?}", controller);
                    state.controllers.insert(controller);
                }
            },
            NotificationType::Type_NodeAdded => {
                let node = notification.get_node();
                let controller = notification.get_controller();
                println!("Added new node: {:?}", node);
                {
                    let mut state = self.state.lock().unwrap();
                    state.nodes.insert(node);
                    let nodes_vec = state.nodes_map.entry(controller).or_insert(Vec::new());
                    nodes_vec.push(node);
                }

            },
            NotificationType::Type_ValueAdded => {
                let value = notification.get_value_id();
                println!("Value added: {:?}", value);
            },
            NotificationType::Type_ValueChanged => {
                let value = notification.get_value_id();
                println!("Value changed: {:?}", value);
            },
            _ => {
                //println!("Unknown notification: {:?}", notification);
            }
        }
    }
}

fn main() {
    let mut options = options::Options::create("./config/", "", "--SaveConfiguration true --DumpTriggerLevel 0 --ConsoleOutput false").unwrap();

    // TODO: The NetworkKey should really be derived from something unique
    //       about the foxbox that we're running on. This particular set of
    //       values happens to be the default that domoticz uses.
    options::Options::add_option_string(&mut options, "NetworkKey", "0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10", false).unwrap();

    let mut manager = manager::Manager::create(options).unwrap();
    let program = Program::new();

    manager.add_watcher(program).unwrap();

    {
        let arg_device: Option<String> = std::env::args()
            .skip(1).last(); // last but not first

        let device = match arg_device {
            Some(ref x) => x as &str,
            None => get_default_device().expect("No device found.")
        };

        println!("found device {}", device);

        match device {
            "usb" => manager.add_usb_driver(),
            _ => manager.add_driver(&device)
        }.unwrap()
    }


    println!("Enter `exit` to exit.");
    let mut input = String::new();
    while input.trim() != "exit" {
        input.clear();
        print!("> ");
        io::stdout().flush().unwrap(); // https://github.com/rust-lang/rust/issues/23818
        io::stdin().read_line(&mut input).ok();
    }
    println!("Exiting...");
}
