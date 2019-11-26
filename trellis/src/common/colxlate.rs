use crate::math::{fmax, fmin};

//static char* colorscheme;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Hsv<T> {
    pub h: T,
    pub s: T,
    pub v: T,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Cmyk<T> {
    pub c: T,
    pub m: T,
    pub y: T,
    pub k: T,
}

fn hsv2rgb(mut h: f64, s: f64, v: f64) -> Rgb<f64> {
    if s <= 0.0 {
        /* achromatic */
        Rgb { r: v, g: v, b: v }
    } else {
        if h >= 1.0 {
            h = 0.0;
        }
        h = 6.0 * h;
        let i: i32 = h as i32;
        let f = h - i as f64;
        let p = v * (1.0 - s);
        let q = v * (1.0 - (s * f));
        let t = v * (1.0 - (s * (1.0 - f)));
        match i {
            0 => Rgb { r: v, g: t, b: p },
            1 => Rgb { r: q, g: v, b: p },
            2 => Rgb { r: p, g: v, b: t },
            3 => Rgb { r: p, g: q, b: v },
            4 => Rgb { r: t, g: p, b: v },
            5 => Rgb { r: v, g: p, b: q },
            _ => panic!("not handled"),
        }
    }
}

fn rgb2hsv(r: f64, g: f64, b: f64) -> Hsv<f64> {
    let mut ht = 0.0;
    let mut st = 0.0;

    let rgbmin = fmin(r, fmin(g, b));
    let rgbmax = fmax(r, fmax(g, b));

    if rgbmax > 0.0 {
        st = (rgbmax - rgbmin) / rgbmax;
    }

    if st > 0.0 {
        let rc = (rgbmax - r) / (rgbmax - rgbmin);
        let gc = (rgbmax - g) / (rgbmax - rgbmin);
        let bc = (rgbmax - b) / (rgbmax - rgbmin);
        if r == rgbmax {
            ht = bc - gc;
        } else if g == rgbmax {
            ht = 2.0 + rc - bc;
        } else if b == rgbmax {
            ht = 4.0 + gc - rc;
        }
        ht = ht * 60.0;
        if ht < 0.0 {
            ht += 360.0;
        }
    }

    Hsv {
        h: ht / 360.0,
        v: rgbmax,
        s: st,
    }
}

fn rgb2cmyk(r: f64, g: f64, b: f64) -> Cmyk<f64> {
    let c = 1.0 - r;
    let m = 1.0 - g;
    let y = 1.0 - b;
    let k = fmin(y, fmin(c, m));
    Cmyk {
        c: c - k,
        m: m - k,
        y: y - k,
        k,
    }
}

/*

static int colorcmpf(const void *p0, const void *p1)
{
    return strcasecmp(((hsvrgbacolor_t *) p0)->name, ((hsvrgbacolor_t *) p1)->name);
}

char *canontoken(char *str)
{
    static unsigned char *canon;
    static size_t allocated;
    unsigned char c, *p, *q;
    size_t len;

    p = (unsigned char *) str;
    len = strlen(str);
    if (len >= allocated) {
    allocated = len + 1 + 10;
    canon = grealloc(canon, allocated);
    if (!canon)
        return NULL;
    }
    q = (unsigned char *) canon;
    while ((c = *p++)) {
    /* if (isalnum(c) == FALSE) */
        /* continue; */
    if (isupper(c))
        c = (unsigned char) tolower(c);
    *q++ = c;
    }
    *q = '\0';
    return (char*)canon;
}

/* fullColor:
 * Return "/prefix/str"
 */
static char* fullColor (char* prefix, char* str)
{
    static char *fulls;
    static size_t allocated;
    size_t len = strlen(prefix) + strlen(str) + 3;

    if (len >= allocated) {
    allocated = len + 10;
    fulls = grealloc(fulls, allocated);
    }
    sprintf (fulls, "/%s/%s", prefix, str);
    return fulls;
}

/* resolveColor:
 * Resolve input color str allowing color scheme namespaces.
 *  0) "black" => "black"
 *     "white" => "white"
 *     "lightgrey" => "lightgrey"
 *    NB: This is something of a hack due to the remaining codegen.
 *        Once these are gone, this case could be removed and all references
 *        to "black" could be replaced by "/X11/black".
 *  1) No initial / =>
 *          if colorscheme is defined and no "X11", return /colorscheme/str
 *          else return str
 *  2) One initial / => return str+1
 *  3) Two initial /'s =>
 *       a) If colorscheme is defined and not "X11", return /colorscheme/(str+2)
 *       b) else return (str+2)
 *  4) Two /'s, not both initial => return str.
 *
 * Note that 1), 2), and 3b) allow the default X11 color scheme.
 *
 * In other words,
 *   xxx => /colorscheme/xxx     if colorscheme is defined and not "X11"
 *   xxx => xxx                  otherwise
 *   /xxx => xxx
 *   /X11/yyy => yyy
 *   /xxx/yyy => /xxx/yyy
 *   //yyy => /colorscheme/yyy   if colorscheme is defined and not "X11"
 *   //yyy => yyy                otherwise
 *
 * At present, no other error checking is done. For example,
 * yyy could be "". This will be caught later.
 */

#define DFLT_SCHEME "X11/"      /* Must have final '/' */
#define DFLT_SCHEME_LEN ((sizeof(DFLT_SCHEME)-1)/sizeof(char))
#define ISNONDFLT(s) ((s) && *(s) && strncasecmp(DFLT_SCHEME, s, DFLT_SCHEME_LEN-1))

static char* resolveColor (char* str)
{
    char* s;
    char* ss;   /* second slash */
    char* c2;   /* second char */

    if (!strcmp(str, "black")) return str;
    if (!strcmp(str, "white")) return str;
    if (!strcmp(str, "lightgrey")) return str;
    if (*str == '/') {   /* if begins with '/' */
    c2 = str+1;
        if ((ss = strchr(c2, '/'))) {  /* if has second '/' */
        if (*c2 == '/') {    /* if second '/' is second character */
            /* Do not compare against final '/' */
        if (ISNONDFLT(colorscheme))
            s = fullColor (colorscheme, c2+1);
        else
            s = c2+1;
        }
        else if (strncasecmp(DFLT_SCHEME, c2, DFLT_SCHEME_LEN)) s = str;
        else s = ss + 1;
    }
    else s = c2;
    }
    else if (ISNONDFLT(colorscheme)) s = fullColor (colorscheme, str);
    else s = str;
    return canontoken(s);
}

int colorxlate(char *str, gvcolor_t * color, color_type_t target_type)
{
    static hsvrgbacolor_t *last;
    static unsigned char *canon;
    static size_t allocated;
    unsigned char *p, *q;
    hsvrgbacolor_t fake;
    unsigned char c;
    f64 H, S, V, A, R, G, B;
    f64 C, M, Y, K;
    unsigned int r, g, b, a;
    size_t len;
    int rc;

    color->type = target_type;

    rc = COLOR_OK;
    for (; *str == ' '; str++);	/* skip over any leading whitespace */
    p = (unsigned char *) str;

    /* test for rgb value such as: "#ff0000"
       or rgba value such as "#ff000080" */
    a = 255;			/* default alpha channel value=opaque in case not supplied */
    if ((*p == '#')
    && (sscanf((char *) p, "#%2x%2x%2x%2x", &r, &g, &b, &a) >= 3)) {
    switch (target_type) {
    case HSVA_DOUBLE:
        R = (f64) r / 255.0;
        G = (f64) g / 255.0;
        B = (f64) b / 255.0;
        A = (f64) a / 255.0;
        rgb2hsv(R, G, B, &H, &S, &V);
        color->u.HSVA[0] = H;
        color->u.HSVA[1] = S;
        color->u.HSVA[2] = V;
        color->u.HSVA[3] = A;
        break;
    case RGBA_BYTE:
        color->u.rgba[0] = r;
        color->u.rgba[1] = g;
        color->u.rgba[2] = b;
        color->u.rgba[3] = a;
        break;
    case CMYK_BYTE:
        R = (f64) r / 255.0;
        G = (f64) g / 255.0;
        B = (f64) b / 255.0;
        rgb2cmyk(R, G, B, &C, &M, &Y, &K);
        color->u.cmyk[0] = (int) C *255;
        color->u.cmyk[1] = (int) M *255;
        color->u.cmyk[2] = (int) Y *255;
        color->u.cmyk[3] = (int) K *255;
        break;
    case RGBA_WORD:
        color->u.rrggbbaa[0] = r * 65535 / 255;
        color->u.rrggbbaa[1] = g * 65535 / 255;
        color->u.rrggbbaa[2] = b * 65535 / 255;
        color->u.rrggbbaa[3] = a * 65535 / 255;
        break;
    case RGBA_DOUBLE:
        color->u.RGBA[0] = (f64) r / 255.0;
        color->u.RGBA[1] = (f64) g / 255.0;
        color->u.RGBA[2] = (f64) b / 255.0;
        color->u.RGBA[3] = (f64) a / 255.0;
        break;
    case COLOR_STRING:
        break;
    case COLOR_INDEX:
        break;
    }
    return rc;
    }

    /* test for hsv value such as: ".6,.5,.3" */
    if (((c = *p) == '.') || isdigit(c)) {
    len = strlen((char*)p);
    if (len >= allocated) {
        allocated = len + 1 + 10;
        canon = grealloc(canon, allocated);
        if (! canon) {
        rc = COLOR_MALLOC_FAIL;
        return rc;
        }
    }
    q = canon;
    while ((c = *p++)) {
        if (c == ',')
        c = ' ';
        *q++ = c;
    }
    *q = '\0';

    if (sscanf((char *) canon, "%lf%lf%lf", &H, &S, &V) == 3) {
        /* clip to reasonable values */
        H = MAX(MIN(H, 1.0), 0.0);
        S = MAX(MIN(S, 1.0), 0.0);
        V = MAX(MIN(V, 1.0), 0.0);
        switch (target_type) {
        case HSVA_DOUBLE:
        color->u.HSVA[0] = H;
        color->u.HSVA[1] = S;
        color->u.HSVA[2] = V;
        color->u.HSVA[3] = 1.0; /* opaque */
        break;
        case RGBA_BYTE:
        hsv2rgb(H, S, V, &R, &G, &B);
        color->u.rgba[0] = (int) (R * 255);
        color->u.rgba[1] = (int) (G * 255);
        color->u.rgba[2] = (int) (B * 255);
        color->u.rgba[3] = 255;	/* opaque */
        break;
        case CMYK_BYTE:
        hsv2rgb(H, S, V, &R, &G, &B);
        rgb2cmyk(R, G, B, &C, &M, &Y, &K);
        color->u.cmyk[0] = (int) C *255;
        color->u.cmyk[1] = (int) M *255;
        color->u.cmyk[2] = (int) Y *255;
        color->u.cmyk[3] = (int) K *255;
        break;
        case RGBA_WORD:
        hsv2rgb(H, S, V, &R, &G, &B);
        color->u.rrggbbaa[0] = (int) (R * 65535);
        color->u.rrggbbaa[1] = (int) (G * 65535);
        color->u.rrggbbaa[2] = (int) (B * 65535);
        color->u.rrggbbaa[3] = 65535;	/* opaque */
        break;
        case RGBA_DOUBLE:
        hsv2rgb(H, S, V, &R, &G, &B);
        color->u.RGBA[0] = R;
        color->u.RGBA[1] = G;
        color->u.RGBA[2] = B;
        color->u.RGBA[3] = 1.0;	/* opaque */
        break;
        case COLOR_STRING:
        break;
        case COLOR_INDEX:
        break;
        }
        return rc;
    }
    }

    /* test for known color name (generic, not renderer specific known names) */
    fake.name = resolveColor(str);
    if (!fake.name)
    return COLOR_MALLOC_FAIL;
    if ((last == NULL)
    || (last->name[0] != fake.name[0])
    || (strcmp(last->name, fake.name))) {
    last = (hsvrgbacolor_t *) bsearch((void *) &fake,
                      (void *) color_lib,
                      sizeof(color_lib) /
                      sizeof(hsvrgbacolor_t), sizeof(fake),
                      colorcmpf);
    }
    if (last != NULL) {
    switch (target_type) {
    case HSVA_DOUBLE:
        color->u.HSVA[0] = ((f64) last->h) / 255.0;
        color->u.HSVA[1] = ((f64) last->s) / 255.0;
        color->u.HSVA[2] = ((f64) last->v) / 255.0;
        color->u.HSVA[3] = ((f64) last->a) / 255.0;
        break;
    case RGBA_BYTE:
        color->u.rgba[0] = last->r;
        color->u.rgba[1] = last->g;
        color->u.rgba[2] = last->b;
        color->u.rgba[3] = last->a;
        break;
    case CMYK_BYTE:
        R = (last->r) / 255.0;
        G = (last->g) / 255.0;
        B = (last->b) / 255.0;
        rgb2cmyk(R, G, B, &C, &M, &Y, &K);
        color->u.cmyk[0] = (int) C * 255;
        color->u.cmyk[1] = (int) M * 255;
        color->u.cmyk[2] = (int) Y * 255;
        color->u.cmyk[3] = (int) K * 255;
        break;
    case RGBA_WORD:
        color->u.rrggbbaa[0] = last->r * 65535 / 255;
        color->u.rrggbbaa[1] = last->g * 65535 / 255;
        color->u.rrggbbaa[2] = last->b * 65535 / 255;
        color->u.rrggbbaa[3] = last->a * 65535 / 255;
        break;
    case RGBA_DOUBLE:
        color->u.RGBA[0] = last->r / 255.0;
        color->u.RGBA[1] = last->g / 255.0;
        color->u.RGBA[2] = last->b / 255.0;
        color->u.RGBA[3] = last->a / 255.0;
        break;
    case COLOR_STRING:
        break;
    case COLOR_INDEX:
        break;
    }
    return rc;
    }

    /* if we're still here then we failed to find a valid color spec */
    rc = COLOR_UNKNOWN;
    switch (target_type) {
    case HSVA_DOUBLE:
    color->u.HSVA[0] = color->u.HSVA[1] = color->u.HSVA[2] = 0.0;
    color->u.HSVA[3] = 1.0; /* opaque */
    break;
    case RGBA_BYTE:
    color->u.rgba[0] = color->u.rgba[1] = color->u.rgba[2] = 0;
    color->u.rgba[3] = 255;	/* opaque */
    break;
    case CMYK_BYTE:
    color->u.cmyk[0] =
        color->u.cmyk[1] = color->u.cmyk[2] = color->u.cmyk[3] = 0;
    break;
    case RGBA_WORD:
    color->u.rrggbbaa[0] = color->u.rrggbbaa[1] = color->u.rrggbbaa[2] = 0;
    color->u.rrggbbaa[3] = 65535;	/* opaque */
    break;
    case RGBA_DOUBLE:
    color->u.RGBA[0] = color->u.RGBA[1] = color->u.RGBA[2] = 0.0;
    color->u.RGBA[3] = 1.0;	/* opaque */
    break;
    case COLOR_STRING:
    break;
    case COLOR_INDEX:
    break;
    }
    return rc;
}

static void rgba_wordToByte (int* rrggbbaa, unsigned char* rgba)
{
    int i;

    for (i = 0; i < 4; i++) {
    rgba[i] = rrggbbaa[i] * 255 / 65535;
    }
}

static void rgba_dblToByte (f64* RGBA, unsigned char* rgba)
{
    int i;

    for (i = 0; i < 4; i++) {
    rgba[i] = (unsigned char)(RGBA[i] * 255);
    }
}

/* colorCvt:
 * Color format converter.
 * Except for the trivial case, it converts the input color to a string
 * representation and then calls colorxlate.
 * ncolor must point to a gvcolor_t struct with type specifying the desired
 * output type.
 */
int colorCvt(gvcolor_t *ocolor, gvcolor_t *ncolor)
{
    int rc;
    char buf[BUFSIZ];
    char* s;
    unsigned char rgba[4];

    if (ocolor->type == ncolor->type) {
    memcpy (&ncolor->u, &ocolor->u, sizeof(ocolor->u));
    return COLOR_OK;
    }
    s = buf;
    switch (ocolor->type) {
    case HSVA_DOUBLE :
    sprintf (buf, "%.03f %.03f %.03f %.03f",
        ocolor->u.HSVA[0], ocolor->u.HSVA[1], ocolor->u.HSVA[2], ocolor->u.HSVA[3]);
    break;
    case RGBA_BYTE :
    sprintf (buf, "#%02x%02x%02x%02x",
        ocolor->u.rgba[0], ocolor->u.rgba[1], ocolor->u.rgba[2], ocolor->u.rgba[3]);
    break;
    case RGBA_WORD:
    rgba_wordToByte (ocolor->u.rrggbbaa, rgba);
    sprintf (buf, "#%02x%02x%02x%02x", rgba[0], rgba[1], rgba[2], rgba[3]);
    break;
    case RGBA_DOUBLE:
    rgba_dblToByte (ocolor->u.RGBA, rgba);
    sprintf (buf, "#%02x%02x%02x%02x", rgba[0], rgba[1], rgba[2], rgba[3]);
    break;
    case COLOR_STRING:
    s = ocolor->u.string;
    break;
    case CMYK_BYTE :
    /* agerr (AGWARN, "Input color type 'CMYK_BYTE' not supported for conversion\n"); */
    return COLOR_UNKNOWN;
    break;
    case COLOR_INDEX:
    /* agerr (AGWARN, "Input color type 'COLOR_INDEX' not supported for conversion\n"); */
    return COLOR_UNKNOWN;
    break;
    default:
    /* agerr (AGWARN, "Unknown input color type value '%u'\n", ncolor->type); */
    return COLOR_UNKNOWN;
    break;
    }
    rc = colorxlate (s, ncolor, ncolor->type);
    return rc;
}

/* setColorScheme:
 * Set current color scheme for resolving names.
 */
void setColorScheme (char* s)
{
    colorscheme = s;
}




*/
