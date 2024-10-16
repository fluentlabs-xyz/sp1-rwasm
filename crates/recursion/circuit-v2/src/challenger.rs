use p3_field::AbstractField;
use sp1_recursion_compiler::{
    circuit::CircuitV2Builder,
    prelude::{Builder, Config, Ext, Felt},
};
use sp1_recursion_core_v2::{
    air::ChallengerPublicValues,
    runtime::{HASH_RATE, PERMUTATION_WIDTH},
    NUM_BITS,
};

// use crate::{DigestVariable, VerifyingKeyVariable};

/// Reference: [p3_challenger::CanObserve].
pub trait CanObserveVariable<C: Config, V> {
    fn observe(&mut self, builder: &mut Builder<C>, value: V);

    fn observe_slice(&mut self, builder: &mut Builder<C>, values: impl IntoIterator<Item = V>) {
        for value in values {
            self.observe(builder, value);
        }
    }
}

pub trait CanSampleVariable<C: Config, V> {
    fn sample(&mut self, builder: &mut Builder<C>) -> V;
}

/// Reference: [p3_challenger::FieldChallenger].
pub trait FieldChallengerVariable<C: Config, Bit>:
    CanObserveVariable<C, Felt<C::F>> + CanSampleVariable<C, Felt<C::F>> + CanSampleBitsVariable<C, Bit>
{
    fn sample_ext(&mut self, builder: &mut Builder<C>) -> Ext<C::F, C::EF>;

    fn check_witness(&mut self, builder: &mut Builder<C>, nb_bits: usize, witness: Felt<C::F>);

    fn duplexing(&mut self, builder: &mut Builder<C>);
}

pub trait CanSampleBitsVariable<C: Config, V> {
    fn sample_bits(&mut self, builder: &mut Builder<C>, nb_bits: usize) -> Vec<V>;
}

/// Reference: [p3_challenger::DuplexChallenger]
#[derive(Clone)]
pub struct DuplexChallengerVariable<C: Config> {
    pub sponge_state: [Felt<C::F>; PERMUTATION_WIDTH],
    pub input_buffer: Vec<Felt<C::F>>,
    pub output_buffer: Vec<Felt<C::F>>,
}

impl<C: Config> DuplexChallengerVariable<C> {
    /// Creates a new duplex challenger with the default state.
    pub fn new(builder: &mut Builder<C>) -> Self {
        DuplexChallengerVariable::<C> {
            sponge_state: core::array::from_fn(|_| builder.eval(C::F::zero())),
            input_buffer: vec![],
            output_buffer: vec![],
        }
    }

    /// Creates a new challenger with the same state as an existing challenger.
    pub fn copy(&self, builder: &mut Builder<C>) -> Self {
        let DuplexChallengerVariable { sponge_state, input_buffer, output_buffer } = self;
        let sponge_state = sponge_state.map(|x| builder.eval(x));
        let mut copy_vec = |v: &Vec<Felt<C::F>>| v.iter().map(|x| builder.eval(*x)).collect();
        DuplexChallengerVariable::<C> {
            sponge_state,
            input_buffer: copy_vec(input_buffer),
            output_buffer: copy_vec(output_buffer),
        }
    }

    // /// Asserts that the state of this challenger is equal to the state of another challenger.
    // fn assert_eq(&self, builder: &mut Builder<C>, other: &Self) {
    //     zip(&self.sponge_state, &other.sponge_state)
    //         .chain(zip(&self.input_buffer, &other.input_buffer))
    //         .chain(zip(&self.output_buffer, &other.output_buffer))
    //         .for_each(|(&element, &other_element)| {
    //             builder.assert_felt_eq(element, other_element);
    //         });
    // }

    // fn reset(&mut self, builder: &mut Builder<C>) {
    //     self.sponge_state.fill(builder.eval(C::F::zero()));
    //     self.input_buffer.clear();
    //     self.output_buffer.clear();
    // }

    fn observe(&mut self, builder: &mut Builder<C>, value: Felt<C::F>) {
        self.output_buffer.clear();

        self.input_buffer.push(value);

        if self.input_buffer.len() == HASH_RATE {
            self.duplexing(builder);
        }
    }

    // fn observe_commitment(&mut self, builder: &mut Builder<C>, commitment: DigestVariable<C>) {
    //     for element in commitment {
    //         self.observe(builder, element);
    //     }
    // }

    fn sample(&mut self, builder: &mut Builder<C>) -> Felt<C::F> {
        if !self.input_buffer.is_empty() || self.output_buffer.is_empty() {
            self.duplexing(builder);
        }

        self.output_buffer.pop().expect("output buffer should be non-empty")
    }

    fn sample_bits(&mut self, builder: &mut Builder<C>, nb_bits: usize) -> Vec<Felt<C::F>> {
        assert!(nb_bits <= NUM_BITS);
        let rand_f = self.sample(builder);
        let mut rand_f_bits = builder.num2bits_v2_f(rand_f, NUM_BITS);
        rand_f_bits.truncate(nb_bits);
        rand_f_bits
    }

    pub fn public_values(&self, builder: &mut Builder<C>) -> ChallengerPublicValues<Felt<C::F>> {
        assert!(self.input_buffer.len() <= PERMUTATION_WIDTH);
        assert!(self.output_buffer.len() <= PERMUTATION_WIDTH);

        let sponge_state = self.sponge_state;
        let num_inputs = builder.eval(C::F::from_canonical_usize(self.input_buffer.len()));
        let num_outputs = builder.eval(C::F::from_canonical_usize(self.output_buffer.len()));

        let input_buffer: [_; PERMUTATION_WIDTH] = self
            .input_buffer
            .iter()
            .copied()
            .chain((self.input_buffer.len()..PERMUTATION_WIDTH).map(|_| builder.eval(C::F::zero())))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let output_buffer: [_; PERMUTATION_WIDTH] = self
            .output_buffer
            .iter()
            .copied()
            .chain(
                (self.output_buffer.len()..PERMUTATION_WIDTH).map(|_| builder.eval(C::F::zero())),
            )
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        ChallengerPublicValues {
            sponge_state,
            num_inputs,
            input_buffer,
            num_outputs,
            output_buffer,
        }
    }
}

impl<C: Config> CanObserveVariable<C, Felt<C::F>> for DuplexChallengerVariable<C> {
    fn observe(&mut self, builder: &mut Builder<C>, value: Felt<C::F>) {
        DuplexChallengerVariable::observe(self, builder, value);
    }

    fn observe_slice(
        &mut self,
        builder: &mut Builder<C>,
        values: impl IntoIterator<Item = Felt<C::F>>,
    ) {
        for value in values {
            self.observe(builder, value);
        }
    }
}

impl<C: Config, const N: usize> CanObserveVariable<C, [Felt<C::F>; N]>
    for DuplexChallengerVariable<C>
{
    fn observe(&mut self, builder: &mut Builder<C>, values: [Felt<C::F>; N]) {
        for value in values {
            self.observe(builder, value);
        }
    }
}

impl<C: Config> CanSampleVariable<C, Felt<C::F>> for DuplexChallengerVariable<C> {
    fn sample(&mut self, builder: &mut Builder<C>) -> Felt<C::F> {
        DuplexChallengerVariable::sample(self, builder)
    }
}

impl<C: Config> CanSampleBitsVariable<C, Felt<C::F>> for DuplexChallengerVariable<C> {
    fn sample_bits(&mut self, builder: &mut Builder<C>, nb_bits: usize) -> Vec<Felt<C::F>> {
        DuplexChallengerVariable::sample_bits(self, builder, nb_bits)
    }
}

impl<C: Config> FieldChallengerVariable<C, Felt<C::F>> for DuplexChallengerVariable<C> {
    fn sample_ext(&mut self, builder: &mut Builder<C>) -> Ext<C::F, C::EF> {
        let a = self.sample(builder);
        let b = self.sample(builder);
        let c = self.sample(builder);
        let d = self.sample(builder);
        builder.ext_from_base_slice(&[a, b, c, d])
    }

    fn check_witness(
        &mut self,
        builder: &mut Builder<C>,
        nb_bits: usize,
        witness: Felt<<C as Config>::F>,
    ) {
        self.observe(builder, witness);
        let element_bits = self.sample_bits(builder, nb_bits);
        for bit in element_bits {
            builder.assert_felt_eq(bit, C::F::zero());
        }
    }

    fn duplexing(&mut self, builder: &mut Builder<C>) {
        assert!(self.input_buffer.len() <= HASH_RATE);

        self.sponge_state[0..self.input_buffer.len()].copy_from_slice(self.input_buffer.as_slice());
        self.input_buffer.clear();

        self.sponge_state = builder.poseidon2_permute_v2(self.sponge_state);

        self.output_buffer.clear();
        self.output_buffer.extend_from_slice(&self.sponge_state);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::utils::tests::run_test_recursion;
    use p3_challenger::{CanObserve, CanSample, FieldChallenger};
    use p3_field::AbstractField;
    use sp1_recursion_compiler::{
        asm::{AsmBuilder, AsmConfig},
        ir::{Ext, ExtConst, Felt},
    };
    use sp1_stark::{baby_bear_poseidon2::BabyBearPoseidon2, StarkGenericConfig};

    use crate::challenger::{DuplexChallengerVariable, FieldChallengerVariable};

    type SC = BabyBearPoseidon2;
    type F = <SC as StarkGenericConfig>::Val;
    type EF = <SC as StarkGenericConfig>::Challenge;

    #[test]
    fn test_compiler_challenger() {
        let config = SC::default();
        let mut challenger = config.challenger();
        challenger.observe(F::one());
        challenger.observe(F::two());
        challenger.observe(F::two());
        challenger.observe(F::two());
        let result: F = challenger.sample();
        println!("expected result: {}", result);
        let result_ef: EF = challenger.sample_ext_element();
        println!("expected result_ef: {}", result_ef);

        let mut builder = AsmBuilder::<F, EF>::default();

        // let width: Var<_> = builder.eval(F::from_canonical_usize(PERMUTATION_WIDTH));
        let mut challenger = DuplexChallengerVariable::<AsmConfig<F, EF>> {
            sponge_state: core::array::from_fn(|_| builder.eval(F::zero())),
            input_buffer: vec![],
            output_buffer: vec![],
        };
        let one: Felt<_> = builder.eval(F::one());
        let two: Felt<_> = builder.eval(F::two());
        // builder.halt();
        challenger.observe(&mut builder, one);
        challenger.observe(&mut builder, two);
        challenger.observe(&mut builder, two);
        challenger.observe(&mut builder, two);
        let element = challenger.sample(&mut builder);
        let element_ef = challenger.sample_ext(&mut builder);

        let expected_result: Felt<_> = builder.eval(result);
        let expected_result_ef: Ext<_, _> = builder.eval(result_ef.cons());
        builder.print_f(element);
        builder.assert_felt_eq(expected_result, element);
        builder.print_e(element_ef);
        builder.assert_ext_eq(expected_result_ef, element_ef);

        // let program = builder.compile_program();
        run_test_recursion(builder.operations, None);
    }
}
