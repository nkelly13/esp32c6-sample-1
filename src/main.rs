use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;

// use esp_idf_svc::nvs::EspDefaultNvsPartition;
mod rgb_led;

use esp_idf_sys as _;

mod wifi;
mod config;

fn create_client() -> anyhow::Result<HttpClient<EspHttpConnection>> {
    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    Ok(HttpClient::wrap(EspHttpConnection::new(&config)?))
}

fn post_test(client: &mut HttpClient<EspHttpConnection>, url: &str) {
    let headers = [("content-type", "application/json")];
    let mut req = client.request(Method::Post, &url, &headers).unwrap();
    println!("request initiated");
    req.write(&"Hello World!".as_bytes()).expect("Send a hello world post body");
    println!("request appended maybe?");
    let res = req.submit().unwrap();
    println!("Request submitted");
    let status = res.status();
    println!("Response status: {status}");

}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    // esp_idf_svc::log::EspLogger::initialize_default();

    // info!("Hello, world!");
    println!("Hello world!");
    let peripherals = match Peripherals::take() {
        Some(x) => x,
        None => {
            return;
        } 
    };
    let sysloop = match EspSystemEventLoop::take(){
        Ok(sysloop) => sysloop,
        Err(_) => {return}
    };


    let mut led = rgb_led::WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0).unwrap();
    led.set_pixel(rgb_led::RGB8::new(5, 50, 5)).expect("set the led to green");

    let app_config = config::CONFIG;
    println!("SSID: {}, PSK: {}", app_config.wifi_ssid, app_config.wifi_psk);

    let _wifi = wifi::wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    ).expect("failed to connect");

    let mut client = create_client().unwrap();
    post_test(&mut client, "https://rustembedded.requestcatcher.com/");
}