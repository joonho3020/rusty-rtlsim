use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

enum Op {
    And, Or, Not, Input, Const(bool),
}

struct Node {
    op: Op,
    inputs: Vec<usize>, // indices into node list
}

fn build_circuit() -> Vec<Node> {
    vec![
        Node { op: Op::Input, inputs: vec![] },           // 0: a
        Node { op: Op::Input, inputs: vec![] },           // 1: b
        Node { op: Op::And,   inputs: vec![0, 1] },       // 2: a & b
        Node { op: Op::Not,   inputs: vec![2] },          // 3: ~(a & b)
    ]
}


fn emit_rtl_to_jit(nodes: &[Node]) -> Box<dyn Fn(u8, u8) -> u8> {
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("jitbuilder");
    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();
    let mut builder_ctx = FunctionBuilderContext::new();

    ctx.func.signature.params.push(AbiParam::new(types::I8)); // a
    ctx.func.signature.params.push(AbiParam::new(types::I8)); // b
    ctx.func.signature.returns.push(AbiParam::new(types::I8)); // output

    let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
    let entry_block = func_builder.create_block();
    func_builder.append_block_params_for_function_params(entry_block);
    func_builder.switch_to_block(entry_block);
    func_builder.seal_block(entry_block);

    let mut cr_values = vec![Value::from_u32(0); nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        cr_values[i] = match node.op {
            Op::Input => {
                // a = 0, b = 1
                func_builder.block_params(entry_block)[i]
            }
            Op::Const(b) => func_builder.ins().iconst(types::I8, b as i64),
            Op::And => {
                let a = cr_values[node.inputs[0]];
                let b = cr_values[node.inputs[1]];
                func_builder.ins().band(a, b)
            }
            Op::Or => {
                let a = cr_values[node.inputs[0]];
                let b = cr_values[node.inputs[1]];
                func_builder.ins().bor(a, b)
            }
            Op::Not => {
                let a = cr_values[node.inputs[0]];
                func_builder.ins().bnot(a)
            }
        };
    }

    let out = cr_values[nodes.len() - 1];

    let mask = func_builder.ins().iconst(types::I8, 1);
    let masked = func_builder.ins().band(out, mask);
    func_builder.ins().return_(&[masked]);
    func_builder.finalize();

    let func_id = module
        .declare_function("rtl", cranelift_module::Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(func_id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    module.finalize_definitions().expect("finalize_definitions");

    let code_ptr = module.get_finalized_function(func_id);
    let func: extern "C" fn(u8, u8) -> u8 = unsafe { std::mem::transmute(code_ptr) };
    Box::new(move |a, b| func(a, b))
}

fn main() {
    let circuit = build_circuit();
    let sim_fn = emit_rtl_to_jit(&circuit);

    for a in 0..=1 {
        for b in 0..=1 {
            let out = sim_fn(a, b);
            println!("NAND({}, {}) = {}", a, b, out);
        }
    }
}
