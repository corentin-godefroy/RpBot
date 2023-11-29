use mongodb::bson::{Bson, bson};
use mongodb::bson::oid::ObjectId;
use crate::constants::{COLLECTABLE_AMOUNT, COLLECTABLE_SECRET, COLLECTABLE_SECRET_AMOUNT, COLLECTABLE_UNIQUE};

#[allow(non_snake_case)]
pub struct Collectables {
    COLLECTABLE_ITEM_ID : ObjectId,
    COLLECTABLE_AMOUNT : u64,
    COLLECTABLE_UNIQUE : bool,
    COLLECTABLE_SECRET : bool,
    COLLECTABLE_SECRET_AMOUNT : bool
}

impl Into<Bson> for Collectables{
    fn into(self) -> Bson {
        bson!({
            COLLECTABLE_AMOUNT : self.COLLECTABLE_AMOUNT as i64,
            COLLECTABLE_AMOUNT : self.COLLECTABLE_AMOUNT as i64,
            COLLECTABLE_UNIQUE : self.COLLECTABLE_UNIQUE,
            COLLECTABLE_SECRET : self.COLLECTABLE_SECRET,
            COLLECTABLE_SECRET_AMOUNT : self.COLLECTABLE_SECRET_AMOUNT
        })
    }
}