pub struct hsvrgbacolor_t {
    pub name: String,

    pub h: u8,
    pub s: u8,
    pub v: u8,

    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/* possible representations of color in gvcolor_t */
#[derive(Clone, Debug)]
pub enum color_type_t {
    HSVA_DOUBLE([f64; 4]),
    RGBA_BYTE([u8; 4]),
    RGBA_WORD([i32; 4]),
    CMYK_BYTE([u8; 4]),
    RGBA_DOUBLE([f64; 4]),
    COLOR_STRING(String),
    COLOR_INDEX(u32),
}

pub type gvcolor_t = color_type_t;

/*
/* gvcolor_t can hold a color spec in a choice or representations */
pub struct gvcolor_t {
    union {
    double RGBA[4];
    double HSVA[4];
    unsigned char rgba[4];
    unsigned char cmyk[4];
    int rrggbbaa[4];
    char *string;
    int index;
    } u;
    color_type_t type;
}
*/

pub type color_s = gvcolor_t;

//#define COLOR_UNKNOWN 1
//#define COLOR_OK 0
