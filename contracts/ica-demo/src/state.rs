use crate::msg::IcaMsg;
use cw_storage_plus::Item;

pub const ICA_HISTORY: Item<Vec<IcaMsg>> = Item::new("ica_history");
