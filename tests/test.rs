#![feature(plugin)]
#![plugin(png2gba)]

mod results;

#[cfg(test)]
mod tests {
    use results::*;

    #[test]
    fn data_notile() {
        const RESULT_NOTILE: &[u16] = include_image!("test.png");
        assert_eq!(DATA_NOTILE, RESULT_NOTILE);
    }
    
    #[test]
    fn data_tile() {
        const RESULT_TILE: &[u16] = include_image!("test.png", "t");
        assert_eq!(DATA_TILE, RESULT_TILE);
    }
}
