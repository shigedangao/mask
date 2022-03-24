pub mod proto_common {
    tonic::include_proto!("common");
}

use proto_common::CommonInput;
use crate::pcr::common::CommonInput as PCommonInput;
use crate::positivity::common::CommonInput as PosCommonInput;
use utils::Date;

impl From<PCommonInput> for CommonInput {
    fn from(t: PCommonInput) -> Self {
        CommonInput { day: t.day, month: t.month, year: t.year }
    }
}

impl From<PosCommonInput> for CommonInput {
    fn from(t: PosCommonInput) -> Self {
        CommonInput { day: t.day, month: t.month, year: t.year }
    }
}


impl Date for CommonInput {
    fn get_year(&self) -> i32 {
        self.year
    }

    fn get_month(&self) -> i32 {
        self.month
    }

    fn get_day(&self) -> Option<i32> {
        self.day
    }
}
