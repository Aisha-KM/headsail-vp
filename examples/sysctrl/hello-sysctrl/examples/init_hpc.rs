#![no_std]
#![no_main]

use core::ptr;

use headsail_bsp::{pac, rt::entry, sdram, sysctrl::soc_ctrl, ufmt};
use hello_sysctrl::{print_example_name, sprint, sprintln};

const HPC_BASE_ADDR: usize = 0xFFE00000;
const BOOTRAM_OFFSET: usize = 0x10000;
const HPC_BOOTRAM_ADDR: usize = HPC_BASE_ADDR + BOOTRAM_OFFSET;

/// Verbose, standard formatting for ever register access, for maximum visibility during the
/// hardware debug process
///
/// # Parameters
///
/// * `intent_message` - Message to identify what are we *trying* to accomplish with this operation
/// * `f` - The function to perform the changes using the pattern. Must return what was in the register before.
///
/// # Type parameters
///
/// * `V` - The type of the register: what goes in & what comes out
fn verbose_call_with_pattern<V: ufmt::uDisplayHex>(
    intent_message: &str,
    f: impl FnOnce(V) -> V,
    pat: V,
    reg_name: &str,
) {
    // Prints e.g.,
    // "> Enable system X with pattern: `SS_EN <- 0x9`... done. Previous value was `0x0`."
    sprint!(
        "> {} with pattern: `{} <- {:#x}`... ",
        intent_message,
        reg_name,
        pat
    );
    let before_value = f(pat);
    sprintln!("done. Previous value was `{:#x}`.", before_value);
}

#[entry]
fn main() -> ! {
    // These lines are necessary to initialize uDMA UART prints for sprint-macro
    soc_ctrl::periph_clk_div_set(0);
    hello_sysctrl::UdmaUart::init();
    print_example_name!();

    // Enable HPC, interconnect, TLP and SDRAM
    let hpc_bit = 1 << 2;
    let icn_bit = 1 << 5;
    let tlp_bit = 1 << 8;
    let sdram_bit = 1 << 3;
    verbose_call_with_pattern(
        "Enable subsystems: HPC, ICN, TLP, SDRAM",
        soc_ctrl::ss_enable,
        hpc_bit | icn_bit | tlp_bit | sdram_bit,
        "SOC_CTRL_SS_EN",
    );

    // Configure HPC and SDRAM clocks
    let hpc_clk_cfg = 0b1001 << 16;
    let sdram_clk_cfg = 0b1001 << 24;
    verbose_call_with_pattern(
        "Configure HPC & SDRAM clocks",
        soc_ctrl::clk1_mask,
        hpc_clk_cfg | sdram_clk_cfg,
        "SOC_CTRL_CLK_CTRL1",
    );

    // Configure ICN clocks
    let icn_clk_cfg = 0b1001 << 8;
    verbose_call_with_pattern(
        "Configure ICN clocks",
        soc_ctrl::clk2_mask,
        icn_clk_cfg,
        "SOC_CTRL_CLK_CTRL2",
    );

    // Configure TLP clocks
    let tlp_clk_cfg = 0b1001;
    verbose_call_with_pattern(
        "Configure TLP clocks",
        soc_ctrl::clk3_mask,
        tlp_clk_cfg,
        "SOC_CTRL_CLK_CTRL3",
    );

    // Enable SDRAM
    let ddr_mode = 0b1;
    verbose_call_with_pattern(
        "Set SDRAM DDR mode",
        sdram::sdram_cfg_axi_ddr_mode_mask,
        ddr_mode,
        "SDRAM_CFG_AXI_DDR_MODE_ADDR",
    );
    let axi_enable = 0b1 << 1;
    verbose_call_with_pattern(
        "Enable AXI for SDRAM",
        sdram::sdram_cfg_axi_enable_mask,
        axi_enable,
        "SDRAM_CFG_AXI_ENABLE_ADDR",
    );

    for i in 0..5 {
        let addr = HPC_BOOTRAM_ADDR + i * 4;
        sprint!("Write 0x6f into {:#x}... ", addr);
        unsafe { ptr::write_volatile(addr as *mut u32, 0x6f) };
        sprint!("Verify written value... ");
        assert_eq!(unsafe { ptr::read_volatile(addr as *const u32) }, 0x6f);
        sprintln!("done.");
    }

    // Obtain register map for HPC & cluster config
    let hpc = unsafe { pac::Hpc::steal() };
    let cluster_cfg = hpc.cluster_config();

    // Configure execute region for SDRAM
    let sdram_region_len = 0x7000_0000;
    verbose_call_with_pattern(
        "Configure execute region for SDRAM",
        |pat| {
            let pval = cluster_cfg.execute_region_length2().read().bits();
            cluster_cfg
                .execute_region_length2()
                .write(|w| unsafe { w.bits(pat) });
            pval
        },
        sdram_region_len,
        "HPC_CLUSTER_CONFIG_EXECUTE_REGION_LENGTH2",
    );
    let (r2start, r2len) = (
        cluster_cfg.execute_region_addr_base2().read().bits(),
        cluster_cfg.execute_region_length2().read().bits(),
    );
    sprintln!(
        "Execute region for SDRAM is now [{:#x}..{:#x}]",
        r2start,
        r2start + r2len
    );

    // Configure cached region for SDRAM
    verbose_call_with_pattern(
        "Configure cached region for SDRAM",
        |pat| {
            let pval = cluster_cfg.cached_region_addr_length0().read().bits();
            cluster_cfg
                .cached_region_addr_length0()
                .write(|w| unsafe { w.bits(pat) });
            pval
        },
        sdram_region_len,
        "HPC_CLUSTER_CONFIG_CACHED_REGION_LENGTH0",
    );
    let (r0start, r0len) = (
        cluster_cfg.cached_region_addr_base0().read().bits(),
        cluster_cfg.cached_region_addr_length0().read().bits(),
    );
    sprintln!(
        "Cached region for SDRAM is now [{:#x}..{:#x}]",
        r0start,
        r0start + r0len
    );

    // Configure execute region for C2C
    let c2c_region_len = 0x2000_0000;
    verbose_call_with_pattern(
        "Configure execute region for C2C",
        |pat| {
            let pval = cluster_cfg.execute_region_length3().read().bits();
            cluster_cfg
                .execute_region_length3()
                .write(|w| unsafe { w.bits(pat) });
            pval
        },
        c2c_region_len,
        "HPC_CLUSTER_CONFIG_EXECUTE_REGION_LENGTH3",
    );
    let (r3start, r3len) = (
        cluster_cfg.execute_region_addr_base3().read().bits(),
        cluster_cfg.execute_region_length3().read().bits(),
    );
    sprintln!(
        "Execute region for C2C is now [{:#x}..{:#x}]",
        r3start,
        r3start + r3len
    );
    // TODO: should also configure cached region length (cachedregionlength1) for C2C but it's not
    // available on the auto-generated memory map as of now

    // Configure execute region for SRAM. Note that there are multiple SRAMs with some holes in
    // between.
    let srams_region_len = 0x10_0000;
    verbose_call_with_pattern(
        "Configure execute region for shared SRAMs",
        |pat| {
            let pval = cluster_cfg.execute_region_length4().read().bits();
            cluster_cfg
                .execute_region_length4()
                .write(|w| unsafe { w.bits(pat) });
            pval
        },
        srams_region_len,
        "HPC_CLUSTER_CONFIG_EXECUTE_REGION_LENGTH4",
    );
    let (r4start, r4len) = (
        cluster_cfg.execute_region_addr_base4().read().bits(),
        cluster_cfg.execute_region_length4().read().bits(),
    );
    sprintln!(
        "Execute region for shared SRAMs is now [{:#x}..{:#x}]",
        r4start,
        r4start + r4len
    );
    // TODO: should also configure cached region length (cachedregionlength2) for SRAM but it's not
    // available on the auto-generated memory map as of now

    // Turn on HPC core #0
    let hpc_core_en = 0b1 << 20;
    verbose_call_with_pattern(
        "Enable HPC core #0",
        soc_ctrl::clk1_mask,
        hpc_core_en,
        "SOC_CTRL_CLK_CTRL1",
    );

    sprintln!("Bootloader (init_hpc) done. Looping in place.");
    loop {
        continue;
    }
}
