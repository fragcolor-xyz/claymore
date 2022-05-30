use crate::ProtoTrait;
use shards::types::{Seq, Table};

pub struct Program {
  pub on_rez: Option<Seq>,
  pub on_derez: Option<Seq>,
  pub advance: Seq,
}

impl ProtoTrait for Program {
  fn distill(traits: &Table) -> Result<Program, &'static str> {
    let on_rez: Option<Seq> = if let Ok(on_rez) = traits.get_fast_static("OnRez\0").try_into() {
      Some(on_rez)
    } else {
      None
    };
    let on_derez: Option<Seq> = if let Ok(on_derez) = traits.get_fast_static("OnDerez\0").try_into() {
      Some(on_derez)
    } else {
      None
    };
    let advance: Seq = traits.get_fast_static("Advance\0").try_into()?;

    Ok(Program {
      on_rez,
      on_derez,
      advance,
    })
  }
}
