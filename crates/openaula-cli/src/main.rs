use openaula_core::DeviceId;

fn main() {
    let supported = [DeviceId::MINI60_HE_PRO, DeviceId::MINI60_HE_PRO_DONGLE];

    println!("aula");
    println!("Supported device IDs:");
    for device in supported {
        println!("  {:04x}:{:04x}", device.vendor_id, device.product_id);
    }
}
