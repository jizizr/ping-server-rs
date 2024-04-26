pub mod model {
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Default, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
    pub struct Target {
        pub method: String,
        pub host: String,
        pub port: Option<u16>,
        pub record_type: String,
    }

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
    pub struct Answer {
        pub error: Option<String>,
        pub success: u16,
        pub fail: u16,
        pub avg_time: f32,
        total_time: f32,
        pub loss: f32,
    }

    impl Answer {
        pub fn new() -> Self {
            Self {
                error: None,
                success: 0,
                fail: 0,
                avg_time: 0.0,
                total_time: 0.0,
                loss: 100.0,
            }
        }

        pub fn add_success(&mut self, duration: Duration) {
            self.success += 1;
            self.total_time += duration.as_secs_f32() * 1000.0;
        }

        pub fn add_fail(&mut self) {
            self.fail += 1;
        }

        pub fn over(mut self, error_msg: String) -> Self {
            if self.success != 0 {
                self.avg_time = self.total_time as f32 / self.success as f32;
                let total = self.success + self.fail;
                self.loss = self.fail as f32 / total as f32 * 100.0;
            } else {
                self.error = Some(error_msg);
            }
            self
        }
    }
}
