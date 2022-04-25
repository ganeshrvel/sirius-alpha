// use crate::common::api_client::sirius_proxima::SIRIUS_PROXIMA_CLIENT;
// use crate::common::errors::api_errors::{ApiClientError, ApiResponseError};
// use crate::common::errors::wifi_errors::WifiError;
// use crate::common::libs::tm1637::mappings::{Brightness, DisplayState};
// use crate::common::libs::tm1637::TM1637;
// use crate::common::models::sirius_proxima_api::Health;
// use crate::constants::default_values::DefaultValues;
// use crate::features::network_feature;
// use crate::{paniq, CommonError, EnvValues, WifiAdaptor};
// use anyhow::private::kind::TraitKind;
// use anyhow::{anyhow, Error};
// use embedded_svc::wifi::{ClientConnectionStatus, ClientIpStatus, ClientStatus, Status, Wifi};
// use esp_idf_sys::c_types::c_uint;
// use std::ptr::null_mut;
// use std::sync::{Arc, Condvar, Mutex};
// use std::thread;
// use std::thread::JoinHandle;
// use std::time::{Duration, Instant};
// use esp_idf_hal::gpio::{InputPin, OutputPin};
//
// #[derive(Clone, Copy)]
// pub struct DisplayFeature {}
//
// pub const STACK_SIZE: usize = 32768_u32 as usize;
//
// impl DisplayFeature {
//     fn run_tm1673_worker(&self, tm: &TM1637) {
//         // let mut ss = shared_state_clone1.lock();
//         // match ss {
//         //     Err(ee) => {
//         //         println!("--7seg thread error {:?}", ee.to_string())
//         //     }
//         //     Ok(mut ssss) => {
//         //         println!("--7seg thread old shared_state {}", *ssss);
//         //
//         //         *ssss = *ssss + 1;
//         //
//         //         println!("--7seg thread new shared_state_clone1 {}", *ssss);
//         //     }
//         // }
//
//         let mut pin_number = 0;
//
//         let t = time_now.now();
//         //println!("--- time_now {:?}", t);
//
//         //todo remove this test block
//         let t = t + Duration::from_secs(3590);
//         //println!("--- new_time_now {:?}", t);
//
//         //todo remove this test block
//
//         let seconds = t.as_secs() % 60;
//         let minutes = (t.as_secs() / 60) % 60;
//         let hours = (t.as_secs() / 60) / 60;
//         // println!("--- duration {:02}:{:02}", minutes, seconds);
//
//         if hours < 1 {
//             let min_sec_t = format!("{:02}{:02}", minutes, seconds);
//
//             show_colon = !show_colon;
//
//             tm.print_string(&min_sec_t, show_colon, None).unwrap();
//
//             thread::sleep(Duration::from_millis(1000))
//         } else {
//             let hour_min_sec_t = format!(
//                 "{}{} {:02}{} {:02}{}",
//                 hours, "h", minutes, "n", seconds, "c"
//             );
//
//             let c = Tm1637BannerAutoScrollConfig {
//                 scroll_min_char_count: tm.display_size + 1,
//                 delay_ms: 750,
//                 min_char_count_to_be_displayed: tm.display_size,
//             };
//             tm.print_string(&hour_min_sec_t, false, Some(&c)).unwrap();
//
//             thread::sleep(Duration::from_millis(2000))
//         }
//     }
//
//     fn new_tm1673() -> anyhow::Result<TM1637<CLK, DIO>>
//     where
//         CLK: OutputPin<Error = E>,
//         DIO: InputPin<Error = E> + OutputPin<Error = E>,
//     {
//         let mut tm = TM1637::new(&mut clk_g27, &mut dio_g13);
//         tm.set_display_state(DisplayState::On);
//         tm.set_brightness(Brightness::L7);
//         tm.clear()?;
//
//         tm
//     }
//
//     pub fn start_tm1637_thread(this: &Arc<Mutex<Self>>) {
//         let time_now = esp_idf_svc::systime::EspSystemTime {};
//
//         thread::Builder::new() /*.stack_size(32768)*/
//             .spawn(move || {
//                 let timeout_duration = Duration::from_millis(1000);
//                 let mut last_exec_time = Instant::now();
//
//                 let mut this = this
//                     .lock()
//                     .map_err(|e| CommonError::MutexGuard("E0027".to_owned(), e.to_string()))?;
//
//                 let tm = this.new_tm1673();
//                 let mut show_colon = false;
//
//                 loop {
//                     log::debug!("[start_tm1637_thread] entering into the next iteration...");
//
//                     if Instant::now() - last_exec_time
//                         >= Duration::from_millis(DefaultValues::TM1637_THREAD)
//                     {
//                         this.run_tm1673_worker();
//
//                         last_exec_time = Instant::now();
//                     } else {
//                         log::debug!(
//                             "[start_tm1637_thread] the threshold delay not completed yet, \
//                             skipping this iteration..."
//                         );
//                     }
//                 }
//             });
//     }
//
//     pub const fn new() -> Self {
//         Self {}
//     }
// }
