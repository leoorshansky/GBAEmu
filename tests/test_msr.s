.section .text
@ MSR test -- change to user mode
MSR CPSR_c, #0b10000
@ MRS test
MRS R0, CPSR
MOV R1, R0
@ MSR test - set all flags
MSR CPSR_f, #4026531840
MOVEQ R1, #69
MOVCS R1, #70
MOVVS R1, #71
MOVMI R1, #72
