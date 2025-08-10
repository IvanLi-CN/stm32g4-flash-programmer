/* Linker script for STM32G431CB with Flash Programming Buffer */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* STM32G431CB has 128KB Flash and 32KB RAM */
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  
  /* Main RAM for program use */
  RAM : ORIGIN = 0x20000000, LENGTH = 30K
  
  /* Dedicated 2KB buffer for Flash programming communication */
  /* This buffer is used for probe-rs <-> STM32 data transfer */
  FLASH_BUFFER : ORIGIN = 0x20007800, LENGTH = 2K
}

/* Place the flash buffer in a special section */
SECTIONS
{
  .flash_buffer (NOLOAD) : ALIGN(4)
  {
    _flash_buffer_start = .;
    . = . + 2048;
    _flash_buffer_end = .;
  } > FLASH_BUFFER
}

/* Export buffer symbols for use in Rust code */
PROVIDE(_flash_buffer_start = ORIGIN(FLASH_BUFFER));
PROVIDE(_flash_buffer_end = ORIGIN(FLASH_BUFFER) + LENGTH(FLASH_BUFFER));
