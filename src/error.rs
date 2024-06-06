use crate::NineSaves;
use anyhow::Result;

impl NineSaves {
    pub fn try_refresh(&mut self) {
        let result = self.data.refresh();
        self.handle_error(result);
    }
    pub fn handle_error(&mut self, result: Result<()>) {
        if let Err(e) = result {
            self.error_status = Some(format!("{:?}", e));
        }
    }
}
