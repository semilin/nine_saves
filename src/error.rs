use crate::NineSaves;

impl NineSaves {
    pub fn try_refresh(&mut self) {
        let res = self.data.refresh();
        if let Err(e) = res {
            self.error_status = Some(format!("{:?}", e));
        }
    }
}
