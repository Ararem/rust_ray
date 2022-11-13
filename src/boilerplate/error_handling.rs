use color_eyre::eyre;

pub fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()
}