use crate::ProtoTrait;
use chainblocks::types::{BlockRef, Table};

pub struct Program {
  pub on_rez: BlockRef,
  pub on_derez: BlockRef,
  pub advance: BlockRef,
}

impl ProtoTrait for Program {
  fn distill(traits: &Table) -> Result<Program, &'static str> {
    let program: Table = traits.get_fast_static("Program\0").try_into()?;

    let on_rez: BlockRef = program.get_fast_static("OnRez\0").try_into()?;
    let on_derez: BlockRef = program.get_fast_static("OnDerez\0").try_into()?;
    let advance: BlockRef = program.get_fast_static("Advance\0").try_into()?;

    Ok(Program {
      on_rez,
      on_derez,
      advance,
    })
  }
}
