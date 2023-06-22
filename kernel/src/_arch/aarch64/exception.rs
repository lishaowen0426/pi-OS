use crate::{errno::ErrorCode, exception::PrivilegeLevel, interrupt::IRQ_CONTROLLER, println};
use aarch64_cpu::{asm::barrier, registers::*};
use core::{arch::asm, fmt};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    registers::InMemoryRegister,
};

extern "C" {
    static __exception_vector_start: u8;
}

pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
    let el = CurrentEL.read_as_enum(CurrentEL::EL);
    match el {
        Some(CurrentEL::EL::Value::EL2) => (PrivilegeLevel::Hypervisor, "EL2"),
        Some(CurrentEL::EL::Value::EL1) => (PrivilegeLevel::Kernel, "EL1"),
        Some(CurrentEL::EL::Value::EL0) => (PrivilegeLevel::User, "EL0"),
        _ => (PrivilegeLevel::Unknown, "Unknown"),
    }
}

#[repr(transparent)]
struct SpsrEL1(InMemoryRegister<u64, SPSR_EL1::Register>);
struct EsrEL1(InMemoryRegister<u64, ESR_EL1::Register>);

impl fmt::Display for SpsrEL1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Raw value.
        writeln!(f, "SPSR_EL1 = {:#018x}", self.0.get())?;

        let to_flag_str = |x| -> _ {
            if x {
                "Set"
            } else {
                "Not set"
            }
        };

        writeln!(f, "      Flags:")?;
        writeln!(
            f,
            "            Negative (N): {}",
            to_flag_str(self.0.is_set(SPSR_EL1::N))
        )?;
        writeln!(
            f,
            "            Zero     (Z): {}",
            to_flag_str(self.0.is_set(SPSR_EL1::Z))
        )?;
        writeln!(
            f,
            "            Carry    (C): {}",
            to_flag_str(self.0.is_set(SPSR_EL1::C))
        )?;
        writeln!(
            f,
            "            Overflow (V): {}",
            to_flag_str(self.0.is_set(SPSR_EL1::V))
        )?;

        let to_mask_str = |x| -> _ {
            if x {
                "Masked"
            } else {
                "Unmasked"
            }
        };

        writeln!(f, "      Software step:")?;
        writeln!(f, "            SS  : {}", self.0.read(SPSR_EL1::SS),)?;

        writeln!(f, "      Exception handling state:")?;
        writeln!(
            f,
            "            Debug  (D): {}",
            to_mask_str(self.0.is_set(SPSR_EL1::D))
        )?;
        writeln!(
            f,
            "            SError (A): {}",
            to_mask_str(self.0.is_set(SPSR_EL1::A))
        )?;
        writeln!(
            f,
            "            IRQ    (I): {}",
            to_mask_str(self.0.is_set(SPSR_EL1::I))
        )?;
        writeln!(
            f,
            "            FIQ    (F): {}",
            to_mask_str(self.0.is_set(SPSR_EL1::F))
        )?;

        writeln!(
            f,
            "      Illegal Execution State (IL): {}",
            to_flag_str(self.0.is_set(SPSR_EL1::IL))
        )?;

        let to_source_str = |x| -> _ {
            match x {
                Some(SPSR_EL1::M::Value::EL0t) => "EL0 with SP0",
                Some(SPSR_EL1::M::Value::EL1t) => "EL1 with SP0",
                Some(SPSR_EL1::M::Value::EL1h) => "EL1 with SP1",
                _ => "UNDEFINED",
            }
        };

        write!(
            f,
            "      The exception was taken from {}",
            to_source_str(self.0.read_as_enum(SPSR_EL1::M))
        )
    }
}

impl EsrEL1 {
    #[inline(always)]
    fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
        self.0.read_as_enum(ESR_EL1::EC)
    }
}

impl fmt::Display for EsrEL1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Raw print of whole register.
        writeln!(f, "ESR_EL1: {:#018x}", self.0.get())?;

        // Raw print of exception class.
        write!(
            f,
            "      Exception Class         (EC) : {:#x}",
            self.0.read(ESR_EL1::EC)
        )?;

        // Exception class.
        write!(f, " - ")?;
        match self.exception_class() {
            Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => writeln!(
                f,
                "{} = {:#018x}",
                "Data Abort, current EL, abort address",
                FAR_EL1.get()
            )?,
            Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => writeln!(
                f,
                "{} = {:#018x}",
                "Instruction Abort, current EL, abort address",
                FAR_EL1.get()
            )?,
            Some(ESR_EL1::EC::Value::InstrAbortLowerEL) => writeln!(
                f,
                "{} = {:#018x}",
                "Instruction Abort, lower EL, abort address",
                FAR_EL1.get()
            )?,
            Some(ESR_EL1::EC::Value::SVC64) => {
                writeln!(f, "{}", "SVC instruction execution in AArch64 state")?
            }
            _ => writeln!(f, "N/A")?,
        };

        // Raw print of instruction specific syndrome.
        write!(
            f,
            "      Instr Specific Syndrome (ISS): {:#x}",
            self.0.read(ESR_EL1::ISS)
        )
    }
}

#[repr(C)]
struct ExceptionContext {
    /// General purpose registers
    gpr: [u64; 30],

    /// Link register
    lr: u64,

    elr_el1: u64,
    spsr_el1: SpsrEL1,
    esr_el1: EsrEL1,
}
impl fmt::Display for ExceptionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExceptionContext:")?;

        let alternating = |x| -> _ {
            if x % 2 == 0 {
                "   "
            } else {
                "\n"
            }
        };

        // Print two registers per line.
        for (i, reg) in self.gpr.iter().enumerate() {
            write!(f, "      x{: <2}: {: >#018x}{}", i, reg, alternating(i))?;
        }

        writeln!(f, "TTBR0_EL1 = {:#018x}", TTBR0_EL1.get_baddr())?;
        writeln!(f, "TTBR1_EL1 = {:#018x}", TTBR1_EL1.get_baddr())?;
        writeln!(f, "TCR_EL1 = {:#066b}", TCR_EL1.get())?;
        writeln!(f, "Link Reg = {:#018x}", self.lr)?;
        writeln!(f, "ELR_EL1  = {:#018x}", self.elr_el1)?;
        writeln!(f, "{}", self.spsr_el1)?;
        write!(f, "{}", self.esr_el1)
    }
}
fn default_synchronous_exception_handler(exc: &ExceptionContext) {
    panic!("CPU Synchronous exception {}", exc);
}
fn default_irq_exception_handler(exc: &ExceptionContext) {
    panic!("CPU Interrupt Request {}", exc);
}
fn default_serro_exception_handler(exc: &ExceptionContext) {
    panic!("CPU SErrir {}", exc);
}

// Current, SP_EL0

#[no_mangle]
extern "C" fn current_el0_synchronous(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_irq(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_serror(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

// Current, SP_ELx
#[no_mangle]
extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    default_synchronous_exception_handler(e);
}

#[no_mangle]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    IRQ_CONTROLLER.get().unwrap().handle().unwrap();
}

#[no_mangle]
extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_serro_exception_handler(e);
}

// Lower, AArch64
#[no_mangle]
extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    default_synchronous_exception_handler(e);
}

#[no_mangle]
extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    IRQ_CONTROLLER.get().unwrap().handle().unwrap();
}

#[no_mangle]
extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    default_serro_exception_handler(e);
}
// Lower, AArch32
#[no_mangle]
extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    default_synchronous_exception_handler(e);
}

#[no_mangle]
extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    default_irq_exception_handler(e);
}

#[no_mangle]
extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    default_serro_exception_handler(e);
}

const DAIF_BITS: u8 = 0b0011; // mask IRQ and FIQ

#[inline(always)]
pub fn local_irq_mask() {
    unsafe {
        asm!("msr DAIFSet, {arg}",
            arg = const DAIF_BITS,
            options(nomem, nostack),
        );
    }
}
#[inline(always)]
pub fn local_irq_unmask() {
    unsafe {
        asm!("msr DAIFClr, {arg}",
        arg = const DAIF_BITS,
        options(nomem, nostack),
        );
    }
}
#[inline(always)]
pub fn local_irq_mask_save() -> u64 {
    let saved = DAIF.get();
    local_irq_mask();
    saved
}
#[inline(always)]
pub fn local_irq_restore(daif: u64) {
    DAIF.set(daif)
}

pub fn print_irq() {
    let to_mask_str = |x| -> _ {
        if x {
            "Masked"
        } else {
            "Unmasked"
        }
    };

    println!(
        "            Debug  (D): {}",
        to_mask_str(DAIF.is_set(DAIF::D))
    );
    println!(
        "            SError (A): {}",
        to_mask_str(DAIF.is_set(DAIF::A))
    );
    println!(
        "            IRQ    (I): {}",
        to_mask_str(DAIF.is_set(DAIF::I))
    );
    println!(
        "            FIQ    (F): {}",
        to_mask_str(DAIF.is_set(DAIF::F))
    );
}

pub fn init() -> Result<(), ErrorCode> {
    // we mask all Interrupts when switch to el1 from el2
    // here we unmask them
    // DAIF.modify(DAIF::D::Unmasked + DAIF::A::Unmasked + DAIF::I::Unmasked + DAIF::F::Unmasked);
    // barrier::isb(barrier::SY);
    print_irq();

    Ok(())
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    use tock_registers::{
        interfaces::{ReadWriteable, Readable, Writeable},
        registers::InMemoryRegister,
    };
}
