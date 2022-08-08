// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use crate::utils::are_equal;
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree,
};

pub struct PublicInputs {
    // pub fp: BaseElement,
    // pub ap: BaseElement,
    // pub pc: BaseElement,
    // pub next_fp: BaseElement,
    // pub next_ap: BaseElement,
    // pub next_pc: BaseElement
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        // target.write(self.fp);
        // target.write(self.ap);
        // target.write(self.pc);
        // target.write(self.next_fp);
        // target.write(self.next_ap);
        // target.write(self.next_pc);
    }
}

// FIBONACCI AIR
// ================================================================================================

pub struct PedersenHashAir {
    context: AirContext<BaseElement>
}

impl Air for PedersenHashAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let mut degrees = vec![TransitionConstraintDegree::new(2); 16];
        degrees.push(TransitionConstraintDegree::new(1));
        assert_eq!(TRACE_WIDTH, trace_info.layout().virtual_trace_width());
        let context =
            // Why does Air context require at least 1 assertion?
            AirContext::new(
                trace_info, 
                degrees,
                1,
                options);
        PedersenHashAir {
            context,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let one = E::ONE;
        let two = E::from(2u128);
        let two_to_15 = E::from((1 as u128) << 15);
        let two_to_16 = E::from((1 as u128) << 16);
        let two_to_32 = E::from((1 as u128) << 32);
        let two_to_48 = E::from((1 as u128) << 48);

        let current = frame.current();
        let next = frame.next();
        // expected state width is nb_columns field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        let (
            f_prefix_0,     f_prefix_1,     f_prefix_2,     f_prefix_3,     f_prefix_4,
            f_prefix_5,     f_prefix_6,     f_prefix_7,     f_prefix_8,     f_prefix_9,
            f_prefix_10,    f_prefix_11,    f_prefix_12,    f_prefix_13,    f_prefix_14,
            f_prefix_15,    off_dst,        off_op0,        off_op1,        pc,
            inst,           dst_addr,       dst,            op0_addr,       op0,
            op1_addr,       op1,            ap,             fp,             mul,
            t0,             t1,             res
        ) = (
            current[3], current[4], current[5], current[6], current[7],
            current[8], current[9], current[10], current[11], current[12],
            current[13], current[14], current[15], current[16], current[17],
            current[18], current[19], current[20], current[21], current[2],
            current[22], current[23], current[24], current[25], current[26],
            current[27], current[28], current[0], current[1], current[29],
            current[30], current[31], current[32],
        );

        let (next_ap, next_fp, next_pc) = (next[0], next[1], next[2]);

        // Flag definitions
        let f_dst_reg = f_prefix_0 - two*f_prefix_1;
        let f_op0_reg = f_prefix_1 - two*f_prefix_2;
        let f_op1_imm = f_prefix_2 - two*f_prefix_3;
        let f_op1_fp = f_prefix_3 - two*f_prefix_4;
        let f_op1_ap = f_prefix_4 - two*f_prefix_5;
        let f_res_add = f_prefix_5 - two*f_prefix_6;
        let f_res_mul = f_prefix_6 - two*f_prefix_7;
        let f_pc_jump_abs = f_prefix_7 - two*f_prefix_8;
        let f_pc_jump_rel = f_prefix_8 - two*f_prefix_9;
        let f_pc_jnz = f_prefix_9 - two*f_prefix_10;
        let f_ap_add = f_prefix_10 - two*f_prefix_11;
        let f_ap_add1 = f_prefix_11 - two*f_prefix_12;
        let f_opcode_call = f_prefix_12 - two*f_prefix_13;
        let f_opcode_ret = f_prefix_13 - two*f_prefix_14;
        let f_opcode_assert_eq = f_prefix_14 - two*f_prefix_15;

        let instruction_size = f_op1_imm + one;

        // Instruction unpacking constraints
        let tilde_off_dst = off_dst + two_to_15;
        let tilde_off_op0 = off_op0 + two_to_15;
        let tilde_off_op1 = off_op1 + two_to_15; 
        //result[0] = are_equal(inst, tilde_off_dst + two_to_16*tilde_off_op0 + two_to_32*tilde_off_op1 + two_to_48*f_prefix_0); //c_inst

        // Check ap correctness at result[0]
        result[0] = are_equal(next_ap, ap + f_ap_add*res + f_ap_add1 + f_opcode_call*two);
        result[1] = f_dst_reg * (f_dst_reg - one);
        result[2] = f_op0_reg * (f_op0_reg - one);
        result[3] = f_op1_imm * (f_op1_imm - one);
        result[4] = f_op1_fp * (f_op1_fp - one);
        result[5] = f_op1_ap * (f_op1_ap - one);
        result[6] = f_res_add * (f_res_add - one);
        result[7] = f_res_mul * (f_res_mul - one);
        result[8] = f_pc_jump_abs * (f_pc_jump_abs - one);
        result[9] = f_pc_jump_rel * (f_pc_jump_rel - one);
        result[10] = f_pc_jnz * (f_pc_jnz - one);
        result[11] = f_ap_add * (f_ap_add - one);
        result[12] = f_ap_add1 * (f_ap_add1 - one);
        result[13] = f_opcode_call * (f_opcode_call - one);
        result[14] = f_opcode_ret * (f_opcode_ret - one);
        result[15] = f_opcode_assert_eq * (f_opcode_assert_eq - one);

        result[16] = f_prefix_15;

        // // Operand constraints
        // result[18] = are_equal(dst_addr, f_dst_reg*fp + (one - f_dst_reg)*ap + off_dst);
        // result[19] = are_equal(op0_addr, f_op0_reg*fp + (one - f_op0_reg)*ap + off_op0);
        // result[20] = are_equal(op1_addr, f_op1_imm*pc + f_op1_ap*ap + f_op1_fp*fp + (one - f_op1_imm - f_op1_ap - f_op1_fp)*op0 + off_op1);

        // // ap and fp registers
        // result[21] = are_equal(next_ap, ap + f_ap_add*res + f_ap_add1 + f_opcode_call*two);
        // result[22] = are_equal(next_fp, f_opcode_ret*dst + f_opcode_call*(ap + two) + (one - f_opcode_ret - f_opcode_call)*fp);

        // // pc register
        // result[23] = are_equal(t0, f_pc_jnz*dst);
        // result[24] = are_equal(t1, t0*res);
        // result[25] = (t1 - f_pc_jnz)*(next_pc - (pc + instruction_size));
        // result[26] = t0*(next_pc - (pc + op1)) + (one - f_pc_jnz)*next_pc - ((one - f_pc_jump_abs - f_pc_jump_rel - f_pc_jnz)*(pc + instruction_size) + f_pc_jump_abs*res + f_pc_jump_rel*(pc + res));

        // // Opcodes and res
        // result[27] = are_equal(mul, op0*op1);
        // result[28] = are_equal((one - f_pc_jnz)*res, f_res_add*(op0 + op1) + f_res_mul*mul + (one - f_res_add - f_res_mul - f_pc_jnz)*op1);
        // result[29] = f_opcode_call*(dst - fp);
        // result[30] = f_opcode_call*(op0 - (pc + instruction_size));
        // result[31] = f_opcode_assert_eq*(dst - res);

    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Add a dummy assetion for now.
        vec![
            Assertion::single(0, 0, Self::BaseField::ONE)
        ]
    }
}
