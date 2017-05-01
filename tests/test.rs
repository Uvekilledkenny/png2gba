#![feature(plugin)]
#![plugin(png2gba)]

mod results;

#[cfg(test)]
mod tests {
    use results::*;

    #[test]
    fn data_notile() {
        const RESULT: &[u16] = include_image!("test.png");
        assert_eq!(DATA_NOTILE, RESULT);
    }
    
    #[test]
    fn data_tile() {
        const RESULT: &[u16] = include_image!("test.png", "t");
        assert_eq!(DATA_TILE, RESULT);
    }

    #[test]
    fn data_palette_notile() {
        const RESULT: (&[u8], &[u16]) = include_image_palette!("test.png", "0xff00ff");
        assert_eq!(DATA_PALETTE_NOTILE, RESULT.0);
        assert_eq!(PALETTE_NOTILE, RESULT.1);
    }
    
    #[test]
    fn data_palette_tile() {
        const RESULT: (&[u8], &[u16]) = include_image_palette!("test.png", "0xff00ff", "t");
        assert_eq!(DATA_PALETTE_TILE, RESULT.0);
        assert_eq!(PALETTE_TILE, RESULT.1);
    }
}
