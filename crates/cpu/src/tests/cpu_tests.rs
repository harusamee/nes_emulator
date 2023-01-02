#[cfg(test)]
pub mod tests {
    use crate::Cpu;
    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.a, 0x05);
        assert!(cpu.f.z == false);
        assert!(cpu.f.c == false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.f.z == true);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = Cpu::new();
        cpu.a = 10;
        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.x, 10);
    }

    #[test]
    fn test_lda_zeropage() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0x85, 0x00, 0xa5, 0x00, 0x85, 0x01]);

        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.bus.read8(0), 1);
        assert_eq!(cpu.bus.read8(1), 1);
    }

    #[test]
    fn test_adc_carry_neg_zero() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0x69, 0x44]);

        assert_eq!(cpu.a, 0x04);
        assert!(cpu.f.c);
        assert!(!cpu.f.n);

        cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0x69, 0xC4]);

        assert_eq!(cpu.a, 0x84);
        assert!(cpu.f.c);
        assert!(cpu.f.n);

        cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0x69, 0x40]);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.f.z);
        assert!(cpu.f.c);
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x69, 0x80]);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.f.c);
        assert!(cpu.f.z);
        assert!(cpu.f.v);
        assert!(!cpu.f.n);

        cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x80]);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.f.c);
        assert!(cpu.f.z);
        assert!(!cpu.f.v);
        assert!(!cpu.f.n);

        cpu.load_and_run(vec![0xa9, 0x7e, 0x69, 0x80]);

        assert_eq!(cpu.a, 0xff);
        assert!(!cpu.f.c);
        assert!(!cpu.f.z);
        assert!(!cpu.f.v);
        assert!(cpu.f.n);
    }


    #[test]
    fn test_sbc_overflow() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0xe9, 0x7f]);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.f.c);
        assert!(cpu.f.z);
        assert!(cpu.f.v);
        assert!(!cpu.f.n);

        cpu.load_and_run(vec![0xa9, 0xff, 0xe9, 0x00]);

        assert_eq!(cpu.a, 0xff);
        assert!(cpu.f.c);
        assert!(!cpu.f.z);
        assert!(!cpu.f.v);
        assert!(cpu.f.n);

        cpu.load_and_run(vec![0xa9, 0x00, 0xe9, 0x00]);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.f.c);
        assert!(cpu.f.z);
        assert!(!cpu.f.v);
        assert!(!cpu.f.n);

        cpu.load_and_run(vec![0xa9, 0x7f, 0xe9, 0x80]);

        assert_eq!(cpu.a, 0xff);
        assert!(!cpu.f.c);
        assert!(!cpu.f.z);
        assert!(cpu.f.v);
        assert!(cpu.f.n);
    }

    #[test]
    fn test_jsr_rts() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0x20, 0x06, 0x06, 0xa9, 0x80, 0x00, 0xa2, 0x40, 0x60]);

        assert_eq!(cpu.pc, 0x0606);
        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.x, 0x40);
        assert!(!cpu.f.c);
        assert!(!cpu.f.z);
        assert!(!cpu.f.v);
        assert!(cpu.f.n);
    }

    #[test]
    fn test_bne_back() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa2, 0x03, 0xca, 0xe0, 0x01, 0xd0, 0xfb]);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.x, 0x01);
        assert!(cpu.f.c);
        assert!(cpu.f.z);
        assert!(!cpu.f.v);
        assert!(!cpu.f.n);
    }

    #[test]
    fn test_beq_forward() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa2, 0x03, 0xe0, 0x03, 0xf0, 0x01, 0x00, 0xa9, 0xff]);

        assert_eq!(cpu.a, 0xff);
        assert_eq!(cpu.x, 0x03);
        assert!(cpu.f.c);
        assert!(!cpu.f.z);
        assert!(!cpu.f.v);
        assert!(cpu.f.n);
    }
    
    #[test]
    fn test_pha_pla() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x48, 0x00, 0xa9, 0x10, 0x68]);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.sp, 0xfe);
        assert!(!cpu.f.c);
        assert!(!cpu.f.z);
        assert!(!cpu.f.v);
        assert!(cpu.f.n);

        cpu.run();

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.sp, 0xff);
        assert!(!cpu.f.c);
        assert!(!cpu.f.z);
        assert!(!cpu.f.v);
        assert!(cpu.f.n);
    }
    
    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = Cpu::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = Cpu::new();
        cpu.x = 0xff;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.x, 1)
    }
}
