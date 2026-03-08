pub mod config;
pub mod consumption;
pub mod device;
pub mod error;
pub mod event;
pub mod heater;
<<<<<<< ours
pub mod input;
=======
pub mod heating_mode; // CORE-T4: HeatingMode as domain type
pub mod input;
pub mod network;      // ExponentialBackoff — host-testable (TEST-T7)
>>>>>>> theirs
