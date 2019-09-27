pub const SPI_SPEED_HZ: u32 = 1_000_000;
pub const WIDTH: u8 = 128;
pub const HEIGHT: u8 = 64;
pub const ST7567_PAGESIZE: u8 = 128;

pub const ST7567_DISPOFF: u8 = 0xae; // 0xae: Display OFF (sleep mode) */
pub const ST7567_DISPON: u8 = 0xaf; // 0xaf: Display ON in normal mode */
pub const ST7567_SETSTARTLINE: u8 = 0x40; // 0x40-7f: Set display start line */
pub const ST7567_STARTLINE_MASK: u8 = 0x3f;

pub const ST7567_REG_RATIO: u8 = 0x20;

pub const ST7567_SETPAGESTART: u8 = 0xb0; // 0xb0-b7: Set page start address */
pub const ST7567_PAGESTART_MASK: u8 = 0x07;

pub const ST7567_SETCOLL: u8 = 0x00; // 0x00-0x0f: Set lower column address */
pub const ST7567_COLL_MASK: u8 = 0x0f;
pub const ST7567_SETCOLH: u8 = 0x10; // 0x10-0x1f: Set higher column address */
pub const ST7567_COLH_MASK: u8 = 0x0f;

pub const ST7567_SEG_DIR_NORMAL: u8 = 0xa0; // 0xa0: Column address 0 is mapped to SEG0 */
pub const ST7567_SEG_DIR_REV: u8 = 0xa1; // 0xa1: Column address 128 is mapped to SEG0 */
pub const ST7567_DISPNORMAL: u8 = 0xa6; // 0xa6: Normal display */
pub const ST7567_DISPINVERSE: u8 = 0xa7; // 0xa7: Inverse display */
pub const ST7567_DISPRAM: u8 = 0xa4; // 0xa4: Resume to RAM content display */
pub const ST7567_DISPENTIRE: u8 = 0xa5; // 0xa5: Entire display ON */
pub const ST7567_BIAS_1_9: u8 = 0xa2; // 0xa2: Select BIAS setting 1/9 */
pub const ST7567_BIAS_1_7: u8 = 0xa3; // 0xa3: Select BIAS setting 1/7 */
pub const ST7567_ENTER_RMWMODE: u8 = 0xe0; // 0xe0: Enter the Read Modify Write mode */
pub const ST7567_EXIT_RMWMODE: u8 = 0xee; // 0xee: Leave the Read Modify Write mode */
pub const ST7567_EXIT_SOFTRST: u8 = 0xe2; // 0xe2: Software RESET */
pub const ST7567_SETCOMNORMAL: u8 = 0xc0; // 0xc0: Set COM output direction, normal mode */
pub const ST7567_SETCOMREVERSE: u8 = 0xc8; // 0xc8: Set COM output direction, reverse mode */
pub const ST7567_POWERCTRL_VF: u8 = 0x29; // 0x29: Control built-in power circuit */
pub const ST7567_POWERCTRL_VR: u8 = 0x2a; // 0x2a: Control built-in power circuit */
pub const ST7567_POWERCTRL_VB: u8 = 0x2c; // 0x2c: Control built-in power circuit */
pub const ST7567_POWERCTRL: u8 = 0x2f; // 0x2c: Control built-in power circuit */
pub const ST7567_REG_RES_RR0: u8 = 0x21; // 0x21: Regulation Resistior ratio */
pub const ST7567_REG_RES_RR1: u8 = 0x22; // 0x22: Regulation Resistior ratio */
pub const ST7567_REG_RES_RR2: u8 = 0x24; // 0x24: Regulation Resistior ratio */
pub const ST7567_SETCONTRAST: u8 = 0x81; // 0x81: Set contrast control */
pub const ST7567_SETBOOSTER: u8 = 0xf8; // Set booster level */
pub const ST7567_SETBOOSTER4X: u8 = 0x00; // Set booster level */
pub const ST7567_SETBOOSTER5X: u8 = 0x01; // Set booster level */
pub const ST7567_NOP: u8 = 0xe3; // 0xe3: NOP Command for no operation */
pub const ST7565_STARTBYTES: u8 = 0;
