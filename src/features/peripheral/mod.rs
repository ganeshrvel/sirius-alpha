use crate::{DeviceError, GpioPinValue};
use embedded_hal::digital::v2::OutputPin;
use esp_idf_hal::gpio::{Gpio13, Gpio14, Gpio25, Gpio26, Gpio27, Gpio32, InputOutput, Output};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys::EspError;
use log::error;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug, Clone, Copy)]
pub enum PeripheralKind {
    PowerOnLed(GpioPinValue),
    WifiConnectedLed(GpioPinValue),
    ProximaApiRequestLed(GpioPinValue),
    AlertBuzzer(GpioPinValue),
}

pub type PeripheralTx = Sender<PeripheralKind>;
pub type PeripheralRx = Receiver<PeripheralKind>;

pub struct Peripheral {
    /// clk pin for TM1637
    pub inout_g27: Gpio27<InputOutput>,

    /// data io pin for TM1637
    pub inout_g13: Gpio13<InputOutput>,

    /// Power on led
    pub out_g32: Gpio32<Output>,

    /// Wifi connected led
    pub out_g25: Gpio25<Output>,

    /// Proxima api request led
    pub out_g26: Gpio26<Output>,

    // Alert buzzer
    pub out_g14: Gpio14<Output>,
}

pub struct PeripheralFeatureStartPins {
    /// Power on led
    pub led_g32: Gpio32<Output>,

    /// Wifi connected led
    pub led_g25: Gpio25<Output>,

    /// Proxima api request led
    pub led_g26: Gpio26<Output>,

    // Alert buzzer
    pub buzzer_g14: Gpio14<Output>,
}

impl Peripheral {
    pub fn set_peripheral(peripheral_tx: &PeripheralTx, p: PeripheralKind) {
        let peripheral_tx_res = peripheral_tx.send(p);
        if let Err(err) = peripheral_tx_res {
            error!("[E0032][PeripheralFeature] {}", err.to_string());
        }
    }

    pub fn start(
        pins: PeripheralFeatureStartPins,
        peripheral_rx: PeripheralRx,
    ) -> anyhow::Result<()> {
        let mut led_g32 = pins.led_g32;
        let mut led_g25 = pins.led_g25;
        let mut led_g26 = pins.led_g26;
        let mut buzzer_g14 = pins.buzzer_g14;

        thread::Builder::new().spawn(move || loop {
            match peripheral_rx.recv() {
                Ok(d) => match d {
                    PeripheralKind::PowerOnLed(s) => {
                        if s == GpioPinValue::High {
                            let res: Result<_, EspError> = led_g32.set_high();
                            if let Err(e) = res {
                                error!("[E0034a][PeripheralFeature][led_g32] {}", e.to_string());
                            }
                        } else {
                            let res: Result<_, EspError> = led_g32.set_low();
                            if let Err(e) = res {
                                error!("[E0034b][PeripheralFeature][led_g32] {}", e.to_string());
                            }
                        }
                    }
                    PeripheralKind::WifiConnectedLed(s) => {
                        if s == GpioPinValue::High {
                            let res: Result<_, EspError> = led_g25.set_high();
                            if let Err(e) = res {
                                error!("[E0034c][PeripheralFeature][led_g25] {}", e.to_string());
                            }
                        } else {
                            let res: Result<_, EspError> = led_g25.set_low();
                            if let Err(e) = res {
                                error!("[E0034d][PeripheralFeature][led_g25] {}", e.to_string());
                            }
                        }
                    }
                    PeripheralKind::ProximaApiRequestLed(s) => {
                        if s == GpioPinValue::High {
                            let res: Result<_, EspError> = led_g26.set_high();
                            if let Err(e) = res {
                                error!("[E0034d][PeripheralFeature][led_g26] {}", e.to_string());
                            }
                        } else {
                            let res: Result<_, EspError> = led_g26.set_low();
                            if let Err(e) = res {
                                error!("[E0034e][PeripheralFeature][led_g26] {}", e.to_string());
                            }
                        }
                    }
                    PeripheralKind::AlertBuzzer(s) => {
                        if s == GpioPinValue::High {
                            let res: Result<_, EspError> = buzzer_g14.set_high();
                            if let Err(e) = res {
                                error!("[E0034d][PeripheralFeature][buzzer_g14] {}", e.to_string());
                            }
                        } else {
                            let res: Result<_, EspError> = buzzer_g14.set_low();
                            if let Err(e) = res {
                                error!("[E0034e][PeripheralFeature][buzzer_g14] {}", e.to_string());
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("[E0033][PeripheralFeature][thread] {}", e.to_string());
                }
            }
        })?;

        Ok(())
    }

    pub fn new() -> anyhow::Result<Self> {
        let p = Peripherals::take();

        return match p {
            None => Err(DeviceError::PeripheralPin("E0030b", "'peripherals' is empty").into()),
            Some(per) => {
                let inout_g27 = per.pins.gpio27.into_input_output()?;
                let inout_g13 = per.pins.gpio13.into_input_output()?;
                let out_g32 = per.pins.gpio32.into_output()?;
                let out_g25 = per.pins.gpio25.into_output()?;
                let out_g26 = per.pins.gpio26.into_output()?;
                let out_g14 = per.pins.gpio14.into_output()?;

                let s = Self {
                    inout_g27,
                    inout_g13,
                    out_g32,
                    out_g25,
                    out_g26,
                    out_g14,
                };

                Ok(s)
            }
        };
    }
}
