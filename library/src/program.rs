use crate::ProtoTrait;
use chainblocks::types::{Table, Seq};

pub struct Program {
  pub on_rez: Seq,
  pub on_derez: Seq,
  pub advance: Seq,
}

impl ProtoTrait for Program {
  fn distill(traits: &Table) -> Result<Program, &'static str> {
    let on_rez: Seq = traits.get_fast_static("OnRez\0").try_into()?;
    let on_derez: Seq = traits.get_fast_static("OnDerez\0").try_into()?;
    let advance: Seq = traits.get_fast_static("Advance\0").try_into()?;

    Ok(Program {
      on_rez,
      on_derez,
      advance,
    })
  }
}
