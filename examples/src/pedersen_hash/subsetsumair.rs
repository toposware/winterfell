use super::{
    BaseElement, FieldElement, ProofOptions, prover::{TRACE_WIDTH, PREFIX, CURVE_POINT, SLOPE},
    hash::{get_intial_constant_point, get_constant_points},
    air::ComposableAir,
};
use crate::{utils::ecc::{enforce_point_addition_affine, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH}, pedersen_hash::prover::{MUX_TRACE_LENGTH}};
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree
};

pub struct PublicInputs { }

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
    }
}

pub struct SubsetSumAir<const CONSTANTS_CYCLE_LENGTH: usize> {
    context: AirContext<<SubsetSumAir<CONSTANTS_CYCLE_LENGTH> as Air>::BaseField>,
}

impl<const CONSTANTS_CYCLE_LENGTH: usize>
SubsetSumAir<CONSTANTS_CYCLE_LENGTH> {
    const INITAL_CONSTANT_POINT: [BaseElement; AFFINE_POINT_WIDTH] = get_intial_constant_point();
}

impl<const CONSTANTS_CYCLE_LENGTH: usize>
ComposableAir for SubsetSumAir<CONSTANTS_CYCLE_LENGTH> {
    const NUM_READ_CELLS_CURRENT: usize = TRACE_WIDTH;
    const NUM_READ_CELLS_NEXT: usize = TRACE_WIDTH;
}

impl<const CONSTANTS_CYCLE_LENGTH: usize>
Air for SubsetSumAir<CONSTANTS_CYCLE_LENGTH> {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        // This checks could be performed at compile time
        assert!(CONSTANTS_CYCLE_LENGTH.is_power_of_two());
        
        let mut degrees = vec![TransitionConstraintDegree::new(2)];
        degrees.append(&mut vec![TransitionConstraintDegree::new(2); 6]);
        degrees.append(&mut vec![TransitionConstraintDegree::new(2); 12]); // TODO: Check

        let context =
        // Why does Air context require at least 1 assertion?
        AirContext::new(
            trace_info, 
            degrees,
            12,
            options);
        SubsetSumAir {
            context,
        }
    }
    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let bit = current[PREFIX] - (E::ONE + E::ONE)*next[PREFIX];
        let lhs = &current[CURVE_POINT];
        let rhs = &periodic_values[..AFFINE_POINT_WIDTH];
        let slope = &current[SLOPE];
        let point = &next[CURVE_POINT];

        // constraint pedersen/hash0/ec_subset_sum/booleanity_test 
        result[0] = bit*(bit - E::ONE);

        // result[0..POINT_COORDINATE_WIDTH] corresponds to constraint pedersen/hash0/ec_subset_sum/add_points/slope
        // result[POINT_COORDINATE_WIDTH .. 2*POINT_COORDINATE_WIDTH] corresponds to pedersen/hash0/ec_subset_sum/add_points/x
        // result[2*POINT_COORDINATE_WIDTH .. 3*POINT_COORDINATE_WIDTH] corresponds to pedersen/hash0/ec_subset_sum/add_points/y
        // TODO: constraints pedersen/hash0/ec_subset_sum/copy_point/x and pedersen/hash0/ec_subset_sum/copy_point/y are missing. 
        enforce_point_addition_affine(
            &mut result[1..],
            lhs,
            rhs,
            slope,
            point,
            bit);

        // Constraints pedersen/hash0/ec_subset_sum/bit_unpacking/ require virtual columns

    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let mut initial_point_assertions = Vec::new();
        for i in 0..AFFINE_POINT_WIDTH{
            initial_point_assertions.push(Assertion::single(i + 1, 0, Self::INITAL_CONSTANT_POINT[i]));
        }
        initial_point_assertions
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut periodic_columns = vec![Vec::new(); AFFINE_POINT_WIDTH];
        for point in get_constant_points::<CONSTANTS_CYCLE_LENGTH>().into_iter() {
            for (i, column) in periodic_columns.iter_mut().enumerate() {
                column.push(point[i]);
            }
        }
        periodic_columns
    }
}