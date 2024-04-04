

fn list_ports() {
    let ports = serialport5::available_ports().expect("No serial ports found!");

    if ports.is_empty() {
        println!("No serial ports found!");
        return;
    }

    println!("Available serial ports ({} ports):", ports.len());
    for (index, port) in ports.iter().enumerate() {
        println!("--- Port #{}: {}", index, port.port_name);
    }
}
