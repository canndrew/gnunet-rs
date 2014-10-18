use ll;

pub struct Ego {
  data: *mut ll::Struct_GNUNET_IDENTITY_Ego,
}

impl Ego {
  pub fn anonymous() -> Ego {
    Ego {
      
    }
  }
}

