SECTIONS
{
    .  = 0x0;         /* From 0x00000000 */

    .text : 
    {
        *(startup)    /* Vector table */
        *(.text)      /* Program code */
    }

    end = .;
}