use mopro_core::middleware::circom;
use mopro_core::middleware::circom::serialization::{SerializableInputs, SerializableProof};
use mopro_core::MoproError;

#[cfg(feature = "gpu-benchmarks")]
use mopro_core::middleware::gpu_explorations::{self, utils::benchmark::BenchmarkResult};

use num_bigint::BigInt;
use std::collections::HashMap;

use std::str::FromStr;
use std::sync::RwLock;

#[cfg(feature = "dylib")]
use std::path::Path;

#[derive(Debug)]
pub enum FFIError {
    MoproError(mopro_core::MoproError),
    SerializationError(String),
}

#[derive(Debug, Clone)]
pub struct GenerateProofResult {
    pub proof: Vec<u8>,
    pub inputs: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct G1 {
    pub x: String,
    pub y: String,
}

#[derive(Debug, Clone)]
pub struct G2 {
    pub x: Vec<String>,
    pub y: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProofCalldata {
    pub a: G1,
    pub b: G2,
    pub c: G1,
}

// NOTE: Need to hardcode the types here, otherwise UniFFI will complain if the gpu-benchmarks feature is not enabled
#[derive(Debug, Clone)]
#[cfg(not(feature = "gpu-benchmarks"))]
pub struct BenchmarkResult {
    pub instance_size: u32,
    pub num_instance: u32,
    pub avg_processing_time: f64,
}

//     pub inputs: Vec<u8>,

impl From<mopro_core::MoproError> for FFIError {
    fn from(error: mopro_core::MoproError) -> Self {
        FFIError::MoproError(error)
    }
}

pub struct MoproCircom {
    state: RwLock<circom::CircomState>,
}

impl Default for MoproCircom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "dylib"))]
pub fn initialize_mopro() -> Result<(), MoproError> {
    // TODO: Error handle / panic?
    circom::initialize();
    Ok(())
}

#[cfg(feature = "dylib")]
pub fn initialize_mopro() -> Result<(), MoproError> {
    println!("need to use dylib to init!");
    panic!("need to use dylib to init!");
}

#[cfg(feature = "dylib")]
pub fn initialize_mopro_dylib(dylib_path: String) -> Result<(), MoproError> {
    // TODO: Error handle / panic?
    let dylib_path = Path::new(dylib_path.as_str());
    circom::initialize(dylib_path);
    Ok(())
}

#[cfg(not(feature = "dylib"))]
pub fn initialize_mopro_dylib(_dylib_path: String) -> Result<(), MoproError> {
    println!("dylib feature not enabled!");
    panic!("dylib feature not enabled!");
}

pub fn generate_proof2(
    inputs: HashMap<String, Vec<String>>,
) -> Result<GenerateProofResult, MoproError> {
    // Convert inputs to BigInt
    let bigint_inputs = inputs
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                v.into_iter()
                    .map(|i| BigInt::from_str(&i).unwrap())
                    .collect(),
            )
        })
        .collect();

    let (proof, inputs) = circom::generate_proof2(bigint_inputs)?;

    let serialized_proof = circom::serialization::serialize_proof(&proof);
    let serialized_inputs = circom::serialization::serialize_inputs(&inputs);
    Ok(GenerateProofResult {
        proof: serialized_proof,
        inputs: serialized_inputs,
    })
}

pub fn full_prove(inputs: HashMap<String, Vec<String>>) -> Result<Vec<String>, MoproError> {
    // Convert inputs to BigInt
    let bigint_inputs = inputs
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                v.into_iter()
                    .map(|i| BigInt::from_str(&i).unwrap())
                    .collect(),
            )
        })
        .collect();

    let witness = circom::full_prove(bigint_inputs).unwrap();

    Ok(witness)
}

pub fn verify_proof2(proof: Vec<u8>, public_input: Vec<u8>) -> Result<bool, MoproError> {
    let deserialized_proof = circom::serialization::deserialize_proof(proof);
    let deserialized_public_input = circom::serialization::deserialize_inputs(public_input);
    let is_valid = circom::verify_proof2(deserialized_proof, deserialized_public_input)?;
    Ok(is_valid)
}

// Convert proof to String-tuples as expected by the Solidity Groth16 Verifier
pub fn to_ethereum_proof(proof: Vec<u8>) -> ProofCalldata {
    let deserialized_proof = circom::serialization::deserialize_proof(proof);
    let proof = circom::serialization::to_ethereum_proof(&deserialized_proof);
    let a = G1 {
        x: proof.a.x.to_string(),
        y: proof.a.y.to_string(),
    };
    let b = G2 {
        x: proof.b.x.iter().map(|x| x.to_string()).collect(),
        y: proof.b.y.iter().map(|x| x.to_string()).collect(),
    };
    let c = G1 {
        x: proof.c.x.to_string(),
        y: proof.c.y.to_string(),
    };
    ProofCalldata { a, b, c }
}

pub fn to_ethereum_inputs(inputs: Vec<u8>) -> Vec<String> {
    let deserialized_inputs = circom::serialization::deserialize_inputs(inputs);
    let inputs = deserialized_inputs
        .0
        .iter()
        .map(|x| x.to_string())
        .collect();
    inputs
}

// TODO: Use FFIError::SerializationError instead
impl MoproCircom {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(circom::CircomState::new()),
        }
    }

    pub fn initialize(&self, zkey_path: String, graph_path: String) -> Result<(), MoproError> {
        let mut state_guard = self.state.write().unwrap();
        state_guard.initialize(zkey_path.as_str(), graph_path.as_str())?;
        Ok(())
    }

    //             inputs: circom::serialization::serialize_inputs(&inputs),
    pub fn generate_witness(
        &self,
        inputs: HashMap<String, Vec<String>>,
    ) -> Result<String, MoproError> {
        let mut state_guard = self.state.write().unwrap();

        // Convert inputs to BigInt
        let bigint_inputs = inputs
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .map(|i| BigInt::from_str(&i).unwrap())
                        .collect(),
                )
            })
            .collect();
        let time = state_guard.generate_witness(bigint_inputs).unwrap();
        Ok(time)
    }

    pub fn generate_proof(
        &self,
    ) -> Result<String, MoproError> {
        let mut state_guard = self.state.write().unwrap();
        let time = state_guard.generate_proof()?;

        Ok(time)
    }

    pub fn verify_proof(&self) -> Result<String, MoproError> {
        let state_guard = self.state.read().unwrap();
        let is_valid = state_guard.verify_proof()?;
        Ok(is_valid.1)
    }
}

#[cfg(feature = "gpu-benchmarks")]
pub fn arkworks_pippenger(
    instance_size: u32,
    num_instance: u32,
    utils_dir: &str,
) -> Result<BenchmarkResult, MoproError> {
    let benchmarks = gpu_explorations::arkworks_pippenger::run_benchmark(
        instance_size,
        num_instance,
        &utils_dir,
    )
    .unwrap();
    Ok(benchmarks)
}
#[cfg(feature = "gpu-benchmarks")]
pub fn trapdoortech_zprize_msm(
    instance_size: u32,
    num_instance: u32,
    utils_dir: &str,
) -> Result<BenchmarkResult, MoproError> {
    let benchmarks = gpu_explorations::trapdoortech_zprize_msm::run_benchmark(
        instance_size,
        num_instance,
        &utils_dir,
    )
    .unwrap();
    Ok(benchmarks)
}

#[cfg(not(feature = "gpu-benchmarks"))]
pub fn arkworks_pippenger(
    instance_size: u32,
    num_instance: u32,
    utils_dir: &str,
) -> Result<BenchmarkResult, MoproError> {
    println!("gpu-benchmarks feature not enabled!");
    Ok(BenchmarkResult {
        instance_size,
        num_instance,
        avg_processing_time: 0.0,
    })
}

#[cfg(not(feature = "gpu-benchmarks"))]
pub fn trapdoortech_zprize_msm(
    instance_size: u32,
    num_instance: u32,
    utils_dir: &str,
) -> Result<BenchmarkResult, MoproError> {
    println!("gpu-benchmarks feature not enabled!");
    Ok(BenchmarkResult {
        instance_size,
        num_instance,
        avg_processing_time: 0.0,
    })
}

fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn hello() -> String {
    "Hello World from Rust".to_string()
}

// TODO: Remove me
// UniFFI expects String type
// See https://mozilla.github.io/uniffi-rs/udl/builtin_types.html
// fn run_example(wasm_path: String, r1cs_path: String) -> Result<(), MoproError> {
//     circom::run_example(wasm_path.as_str(), r1cs_path.as_str())
// }

uniffi::include_scaffolding!("mopro");

#[cfg(test)]
mod tests {
    use core::num;

    use super::*;
    use ark_bn254::Fr;
    use num_bigint::BigUint;

    fn bytes_to_circuit_inputs(input_vec: &Vec<u8>) -> HashMap<String, Vec<String>> {
        let bits = circom::utils::bytes_to_bits(&input_vec);
        let converted_vec: Vec<String> = bits
            .into_iter()
            .map(|bit| (bit as i32).to_string())
            .collect();
        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), converted_vec);
        inputs
    }

    fn bytes_to_circuit_outputs(bytes: &[u8]) -> Vec<u8> {
        let bits = circom::utils::bytes_to_bits(bytes);
        let field_bits = bits.into_iter().map(|bit| Fr::from(bit as u8)).collect();
        let circom_outputs = circom::serialization::SerializableInputs(field_bits);
        circom::serialization::serialize_inputs(&circom_outputs)
    }

    #[test]
    fn add_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    // #[test]
    // fn test_end_to_end() -> Result<(), MoproError> {
    //     // Paths to your wasm and zkey files
    //     let wasm_path =
    //         "./../mopro-core/examples/circom/multiplier2/target/multiplier2_js/multiplier2.wasm";
    //     let zkey_path = "./../mopro-core/examples/circom/multiplier2/target/multiplier2_final.zkey";

    //     // Create a new MoproCircom instance
    //     let mopro_circom = MoproCircom::new();

    //     // Step 1: Initialize
    //     let init_result = mopro_circom.initialize(zkey_path.to_string(), wasm_path.to_string());
    //     assert!(init_result.is_ok());

    //     let mut inputs = HashMap::new();
    //     let a = BigUint::from_str(
    //         "21888242871839275222246405745257275088548364400416034343698204186575808495616",
    //     )
    //     .unwrap();
    //     let b = BigUint::from(1u8);
    //     let c = a.clone() * b.clone();
    //     inputs.insert("a".to_string(), vec![a.to_string()]);
    //     inputs.insert("b".to_string(), vec![b.to_string()]);
    //     // output = [public output c, public input a]
    //     let expected_output = vec![Fr::from(c), Fr::from(a)];
    //     let circom_outputs = circom::serialization::SerializableInputs(expected_output);
    //     let serialized_outputs = circom::serialization::serialize_inputs(&circom_outputs);

    //     // Step 2: Generate Proof
    //     let generate_proof_result = mopro_circom.generate_proof(inputs)?;
    //     let serialized_proof = generate_proof_result.proof;
    //     let serialized_inputs = generate_proof_result.inputs;

    //     assert!(serialized_proof.len() > 0);
    //     assert_eq!(serialized_inputs, serialized_outputs);

    //     // Step 3: Verify Proof
    //     let is_valid =
    //         mopro_circom.verify_proof(serialized_proof.clone(), serialized_inputs.clone())?;
    //     assert!(is_valid);

    //     // Step 4: Convert Proof to Ethereum compatible proof
    //     let proof_calldata = to_ethereum_proof(serialized_proof);
    //     let inputs_calldata = to_ethereum_inputs(serialized_inputs);
    //     assert!(proof_calldata.a.x.len() > 0);
    //     assert!(inputs_calldata.len() > 0);

    //     Ok(())
    // }

    // #[test]
    // fn test_end_to_end_keccak() -> Result<(), MoproError> {
    //     // Paths to your wasm and r1cs files
    //     let wasm_path =
    //         "./../mopro-core/examples/circom/keccak256/target/keccak256_256_test_js/keccak256_256_test.wasm";
    //     let zkey_path =
    //         "./../mopro-core/examples/circom/keccak256/target/keccak256_256_test_final.zkey";

    //     // Create a new MoproCircom instance
    //     let mopro_circom = MoproCircom::new();

    //     // Step 1: Setup
    //     let setup_result = mopro_circom.initialize(zkey_path.to_string(), wasm_path.to_string());
    //     assert!(setup_result.is_ok());

    //     // Prepare inputs
    //     let input_vec = vec![
    //         116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         0, 0, 0, 0, 0, 0,
    //     ];

    //     // Expected output
    //     let expected_output_vec = vec![
    //         37, 17, 98, 135, 161, 178, 88, 97, 125, 150, 143, 65, 228, 211, 170, 133, 153, 9, 88,
    //         212, 4, 212, 175, 238, 249, 210, 214, 116, 170, 85, 45, 21,
    //     ];

    //     let inputs = bytes_to_circuit_inputs(&input_vec);
    //     let serialized_outputs = bytes_to_circuit_outputs(&expected_output_vec);

    //     // Step 2: Generate Proof
    //     let generate_proof_result = mopro_circom.generate_proof(inputs)?;
    //     let serialized_proof = generate_proof_result.proof;
    //     let serialized_inputs = generate_proof_result.inputs;

    //     assert!(serialized_proof.len() > 0);
    //     assert_eq!(serialized_inputs, serialized_outputs);

    //     // Step 3: Verify Proof

    //     let is_valid =
    //         mopro_circom.verify_proof(serialized_proof.clone(), serialized_inputs.clone())?;
    //     assert!(is_valid);

    //     // Step 4: Convert Proof to Ethereum compatible proof
    //     let proof_calldata = to_ethereum_proof(serialized_proof);
    //     let inputs_calldata = to_ethereum_inputs(serialized_inputs);
    //     assert!(proof_calldata.a.x.len() > 0);
    //     assert!(inputs_calldata.len() > 0);

    //     Ok(())
    // }

    // #[test]
    // #[cfg(feature = "gpu-benchmarks")]
    // fn test_arkworks_pippenger() -> Result<(), MoproError> {
    //     let instance_size = 16;
    //     let num_instance = 10;
    //     let utils_dir = "../mopro-core/src/middleware/gpu_explorations/utils/vectors/16x10";
    //     let result = arkworks_pippenger(instance_size, num_instance, &utils_dir).unwrap();
    //     println!("Benchmark result: {:#?}", result);
    //     Ok(())
    // }

    // #[test]
    // #[cfg(feature = "gpu-benchmarks")]
    // fn test_trapdoortech_zprize_msm() -> Result<(), MoproError> {
    //     let instance_size = 16;
    //     let num_instance = 10;
    //     let utils_dir = "../mopro-core/src/middleware/gpu_explorations/utils/vectors/16x10";
    //     let result = trapdoortech_zprize_msm(instance_size, num_instance, utils_dir);
    //     println!("Benchmark result: {:#?}", result);
    //     Ok(())
    // }
}
