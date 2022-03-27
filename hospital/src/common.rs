pub mod proto_common {
    tonic::include_proto!("common");
}

use proto_common::CommonInput;
use crate::hospital::common::CommonInput as HCommonInput;
use crate::icu::common::CommonInput as ICommonInput;
use crate::mix::common::CommonInput as DCommonInput;
use utils::Date;

impl From<HCommonInput> for CommonInput {
    fn from(t: HCommonInput) -> Self {
        CommonInput { day: t.day, month: t.month, year: t.year }
    }
}

impl From<ICommonInput> for CommonInput {
    fn from(t: ICommonInput) -> Self {
        CommonInput { day: t.day, month: t.month, year: t.year }
    }
}

impl From<DCommonInput> for CommonInput {
    fn from(t: DCommonInput) -> Self {
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
