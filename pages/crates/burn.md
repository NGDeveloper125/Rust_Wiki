---
title: "burn"
version: "0.22.0"
publisher: "Nathaniel Simard (nathanielsimard), Sylvain Benner (syl20bnr), Core"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-11"
summary: "A deep learning framework written in Rust: tensors, autodiff, neural network layers, optimizers, a training loop and model storage, with one model definition running on GPUs, CPUs and WebAssembly."
domain: "Machine learning"
categories: ["science", "wasm", "no-std"]
repository: "https://github.com/tracel-ai/burn"
docs: "https://docs.rs/burn/0.22.0/burn/"
---

## Overview

`burn` is a deep learning framework, not a wrapper around one. It supplies the
whole stack in Rust: tensors, automatic differentiation, the usual neural
network layers, optimizers, a training loop with metrics, and weight
storage — and the same model code runs on CUDA, ROCm, Metal, Vulkan, WebGPU,
CPU and WebAssembly.

A model is a plain struct with a derive on it:

```
use burn::nn::{Linear, LinearConfig, Relu};
use burn::prelude::*;

#[derive(Module, Debug)]
struct Mlp {
    hidden: Linear,
    activation: Relu,
    output: Linear,
}

impl Mlp {
    fn new(device: &Device) -> Self {
        Self {
            hidden: LinearConfig::new(4, 8).init(device),
            activation: Relu::new(),
            output: LinearConfig::new(8, 1).init(device),
        }
    }

    fn forward(&self, input: Tensor<2>) -> Tensor<2> {
        let x = self.activation.forward(self.hidden.forward(input));
        self.output.forward(x)
    }
}

let device = Device::default();
let model = Mlp::new(&device);

let output = model.forward(Tensor::zeros([3, 4], &device));
assert_eq!(output.dims(), [3, 1]);
assert_eq!(model.num_params(), 49); // 4*8 + 8 + 8*1 + 1
```

**Read that snippet carefully if you have seen `burn` before, because 0.22
changed its most visible feature.** The backend used to be a type parameter
threaded through everything — `Tensor<B, 2>`, `struct Mlp<B: Backend>`,
`Model<NdArray>`. It is gone. Rank lives in the type (`Tensor<2>`), the backend
is chosen at runtime by the `Device` a tensor is created on, and autodiff is a
decorator on a device rather than a wrapper type. Every tutorial, blog post and
answer written before 0.22 — which is nearly all of them — shows the old form,
and none of it compiles. Check the version before copying anything.

**`burn` ships no backend by default.** The dependency gives you the tensor and
model API and nothing to execute it, and `Device::default()` panics if no
execution backend is compiled in. Pick one with a feature:

`burn = { version = "0.22", features = ["wgpu", "train"] }` for a GPU on
whatever graphics API the platform provides, `["flex", "train"]` for pure-Rust
CPU, or `["cuda", "train"]` on NVIDIA — with `metal`, `vulkan`, `webgpu`,
`rocm` and `cpu` also available, and several usable side by side.

`flex` is the pure-Rust CPU backend and the right default for tests, CI and
`no_std`. Note that the two backends most older material reaches for are now
**deprecated**: `ndarray` is superseded by `flex`, and `tch` by the CubeCL
backends.

**Where it stands against the alternatives.** PyTorch is where the research
happens, where pretrained weights are published and where almost every tutorial
points, and nothing here changes that. `burn`'s case is deployment: no Python
runtime to ship, no `libtorch` to install, a single binary that cross-compiles,
and targets Python cannot reach — `no_std`, embedded, WebAssembly in a browser.
It can import PyTorch checkpoints and SafeTensors files, so training elsewhere
and serving in Rust is a supported path. For comparison,
[`candle`](https://docs.rs/candle-core) is smaller and inference-leaning, and
`tch` binds libtorch directly if what you want *is* PyTorch.

Two practical constraints before you commit. The crate requires **Rust 1.95 and
edition 2024**, which is aggressive enough to rule it out on a pinned
toolchain. And it is **pre-1.0**: the API moves between minor versions, as the
0.22 backend change shows, so expect to do real work at each upgrade.

## When to use it

### Use case: Training a model without writing the loop by hand

The explicit loop — forward, loss, backward, optimizer step — which is also the
clearest way to see what the framework is doing.

```
use burn::nn::loss::{MseLoss, Reduction};
use burn::nn::{Linear, LinearConfig};
use burn::optim::{AdamConfig, GradientsParams};
use burn::prelude::*;

#[derive(Module, Debug)]
struct Model {
    layer: Linear,
}

impl Model {
    fn new(device: &Device) -> Self {
        Self { layer: LinearConfig::new(2, 1).init(device) }
    }

    fn forward(&self, x: Tensor<2>) -> Tensor<2> {
        self.layer.forward(x)
    }
}

// .autodiff() is what makes gradients available; without it backward() has
// nothing to record.
let device = Device::default().autodiff();
let mut model = Model::new(&device);
let mut optimizer = AdamConfig::new().init();

let inputs = Tensor::<2>::from_floats([[1.0, 2.0], [3.0, 4.0]], &device);
let targets = Tensor::<2>::from_floats([[3.0], [7.0]], &device);

let mut first_loss = 0.0f32;
for step in 0..50 {
    let predictions = model.forward(inputs.clone());
    let loss = MseLoss::new().forward(predictions, targets.clone(), Reduction::Mean);

    if step == 0 {
        first_loss = loss.clone().into_scalar();
    }

    // Gradients are matched back to the module's parameters by path.
    let gradients = GradientsParams::from_grads(loss.backward(), &model);
    model = optimizer.step(0.05, model, gradients);
}

let final_loss: f32 = MseLoss::new()
    .forward(model.forward(inputs), targets, Reduction::Mean)
    .into_scalar();

assert!(first_loss > 10.0, "started at {first_loss}");
assert!(final_loss < 1.0, "ended at {final_loss}");
```

**Why it fits:** every piece is visible, which matters when you are learning the
framework or doing something its training harness does not anticipate —
adversarial training, a custom gradient surgery step, reinforcement learning
with an environment in the loop. Note that `optimizer.step` takes the model by
value and returns it: parameters are owned data, not interior-mutable state, so
an update is a transformation rather than a side effect.

### Use case: Saving and reloading trained weights

Training is only useful if the result outlives the process.

```
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;
use burn::store::{BurnpackStore, ModuleSnapshot};

#[derive(Module, Debug)]
struct Net {
    layer: Linear,
}

let device = Device::default();
let trained = Net { layer: LinearConfig::new(2, 2).init(&device) };
let original: Vec<f32> = trained.layer.weight.val().into_data().try_to_vec().unwrap();

let path = std::env::temp_dir().join("burn-page-example.bpk");
let mut out = BurnpackStore::from_file(&path);
trained.save_into(&mut out).unwrap();

// A freshly initialised model has different random weights...
let mut restored = Net { layer: LinearConfig::new(2, 2).init(&device) };
let mut input = BurnpackStore::from_file(&path);
let result = restored.load_from(&mut input).unwrap();

// ...until the saved tensors are applied over it, matched by name.
assert_eq!(result.applied.len(), 2); // weight and bias
let loaded: Vec<f32> = restored.layer.weight.val().into_data().try_to_vec().unwrap();
assert_eq!(original, loaded);
```

**Why it fits:** loading reports what happened rather than succeeding silently —
the `ApplyResult` lists which tensors were applied, which were skipped and which
in the file matched nothing. That is what you want when the saved file came from
a different version of the model, or from PyTorch, where a rename turns into
silently untrained weights if nobody checks. `SafetensorsStore` and
`PytorchStore` read the formats their names suggest.

### Use case: Inference in the browser or on a device without an OS

The reason to pick a Rust framework over calling Python.

```
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;

#[derive(Module, Debug)]
struct Classifier {
    layer: Linear,
}

// No training here: load weights, run forward, return a decision.
let device = Device::default();
let model = Classifier { layer: LinearConfig::new(3, 2).init(&device) };

let batch = Tensor::<2>::from_floats([[0.1, 0.2, 0.3], [0.9, 0.8, 0.7]], &device);
let logits = model.forward_classify(batch);

assert_eq!(logits.dims(), [2, 2]);

// argmax over the class dimension gives the predicted label per row.
let predictions: Vec<i32> = logits.argmax(1).into_data().try_to_vec().unwrap();
assert_eq!(predictions.len(), 2);

impl Classifier {
    fn forward_classify(&self, input: Tensor<2>) -> Tensor<2> {
        self.layer.forward(input)
    }
}
```

**Why it fits:** this compiles to WebAssembly and to `no_std` targets, because
the tensor API and the `flex` backend are both available without an operating
system. There is no interpreter to embed, no `libtorch` to ship beside the
binary, and no FFI boundary to marshal across. That combination is the thing
`burn` offers that the Python ecosystem structurally cannot.

## API map

The crate is a façade over about a dozen sub-crates; `burn::prelude` re-exports
what nearly every program needs. Entries below are grouped by the four jobs:
make tensors, define a model, train it, store it.

### Tensors

#### `Tensor` construction

Rank in the type, device at the call site.

```
use burn::prelude::*;

let device = Device::default();

// From literals, with the rank as a const parameter.
let a = Tensor::<2>::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], &device);
assert_eq!(a.dims(), [2, 3]);

// Filled constructors take a shape.
let zeros = Tensor::<2>::zeros([2, 3], &device);
let ones = Tensor::<1>::ones([4], &device);
assert_eq!(zeros.dims(), [2, 3]);
assert_eq!(ones.dims(), [4]);

// Int and Bool are kinds, the second type parameter.
let ints = Tensor::<1, Int>::from_ints([1, 2, 3], &device);
let mask = ints.greater_elem(1);
let flags: Vec<bool> = mask.into_data().try_to_vec().unwrap();
assert_eq!(flags, vec![false, true, true]);
```

**When to use it:** `Tensor<D>` is `Tensor<D, Float>`; `Int` and `Bool` are the
other two kinds, and the kind decides which operations exist — you cannot
`matmul` a boolean mask. Rank being a const generic means shape mistakes that
PyTorch finds at runtime are often compile errors here, though the *sizes*
within a rank are still checked at run time.

#### Arithmetic and reductions

The operations, all consuming `self`.

```
use burn::prelude::*;

let device = Device::default();
let a = Tensor::<2>::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], &device);
let b = Tensor::<2>::from_floats([[1.0, 0.0], [0.0, 1.0], [1.0, 1.0]], &device);

// Matrix multiply: [2,3] @ [3,2] -> [2,2]
let product = a.clone().matmul(b);
let values: Vec<f32> = product.into_data().try_to_vec().unwrap();
assert_eq!(values, vec![4.0, 5.0, 10.0, 11.0]);

// Elementwise, then a full reduction to a scalar.
let total: f32 = a.clone().mul_scalar(2.0).sum().into_scalar();
assert_eq!(total, 42.0);

// Reductions along one axis keep the other.
let per_row = a.sum_dim(1);
assert_eq!(per_row.dims(), [2, 1]);
```

**When to use it:** everywhere. The thing to internalise is that operations take
`self` by value, so a tensor you need twice gets a `.clone()` — which is cheap,
because the underlying buffer is reference-counted and cloning does not copy
data. The `.clone()` calls littering `burn` code are bookkeeping, not a
performance problem.

#### Reshaping, slicing and reading values out

Getting data into a different shape, or out of the framework entirely.

```
use burn::prelude::*;

let device = Device::default();
let a = Tensor::<2>::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], &device);

let reshaped = a.clone().reshape([3, 2]);
assert_eq!(reshaped.dims(), [3, 2]);

// s! builds a slice specification: all rows, first two columns.
let sliced = a.clone().slice(s![.., 0..2]);
assert_eq!(sliced.dims(), [2, 2]);

// into_data moves to the host; try_to_vec then types it.
let flat: Vec<f32> = a.into_data().try_to_vec().unwrap();
assert_eq!(flat, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
```

**When to use it:** `reshape` and `slice` for the usual tensor plumbing;
`into_data` only at a boundary, because it synchronises with the device and
copies to host memory — doing it inside a training loop to log a value is a
common way to make training mysteriously slow. Use `try_to_vec` rather than
`to_vec`, which is deprecated in 0.22, and give the element type explicitly: it
errors rather than converting if the tensor's dtype differs, which is how you
discover that `argmax` returns `i32`.

### Defining a model

#### `#[derive(Module)]`

Turns a struct into something trainable.

```
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;

#[derive(Module, Debug)]
struct TwoLayer {
    first: Linear,
    second: Linear,
}

let device = Device::default();
let model = TwoLayer {
    first: LinearConfig::new(10, 4).init(&device),
    second: LinearConfig::new(4, 2).init(&device),
};

// The derive walks the fields: 10*4+4 + 4*2+2 = 54
assert_eq!(model.num_params(), 54);
```

**When to use it:** on every model and every reusable block. The derive is what
makes a struct's parameters discoverable — it generates the traversal that
collects gradients, moves parameters between devices, and names each tensor for
saving. Nesting works, so a block is a module with modules in it, and the
generated paths (`first.weight`) are the names used in saved files.

#### `nn` layers and their configs

Layers are built by a config, not a constructor.

```
use burn::nn::{BatchNormConfig, Dropout, DropoutConfig, LinearConfig, Relu};
use burn::prelude::*;

let device = Device::default();

// Required arguments to new, optional ones as with_*.
let linear = LinearConfig::new(8, 4).with_bias(false).init(&device);
let norm = BatchNormConfig::new(4).init(&device);
let dropout: Dropout = DropoutConfig::new(0.5).init();
let activation = Relu::new();

let x = Tensor::<2>::zeros([2, 8], &device);
let y = activation.forward(linear.forward(x));
let y = dropout.forward(norm.forward(y));

assert_eq!(y.dims(), [2, 4]);
```

**When to use it:** the `Config`/`init` split exists because a config is plain
serializable data while the layer owns device-resident tensors — so you can
record the architecture in a file and rebuild it anywhere. Layers that hold no
parameters, like `Relu`, skip the config and have a plain `new`; `Dropout`'s
`init` takes no device for the same reason.

#### `#[derive(Config)]`

Serializable settings with defaults and builders.

```
use burn::config::Config;

#[derive(Config, Debug)]
struct TrainingConfig {
    hidden_size: usize,
    #[config(default = 0.001)]
    learning_rate: f64,
    #[config(default = 32)]
    batch_size: usize,
}

// Fields without a default become arguments to new; the rest get with_*.
let config = TrainingConfig::new(128).with_learning_rate(0.01);

assert_eq!(config.hidden_size, 128);
assert_eq!(config.learning_rate, 0.01);
assert_eq!(config.batch_size, 32);
```

**When to use it:** for hyperparameters and model shape. The derive adds `serde`
impls plus `save`/`load` to JSON, which is how a training run records what it
ran — worth doing from the first experiment, because reproducing a result six
months later from a number hard-coded in a loop is unpleasant.

### Training

#### `Device::autodiff` and `backward`

Gradients are a property of the device.

```
use burn::prelude::*;

// Without .autodiff(), there is no tape and backward() has nothing to give.
let device = Device::default().autodiff();

let x = Tensor::<1>::from_floats([3.0], &device).require_grad();
let y = x.clone().mul(x.clone()); // y = x^2, so dy/dx = 2x = 6

let gradients = y.backward();
let dx = x.grad(&gradients).unwrap();
let value: f32 = dx.into_scalar();

assert_eq!(value, 6.0);
```

**When to use it:** wrap the device once, at the point you build the model for
training, and leave inference devices plain — the tape costs memory and time
that inference has no use for. `require_grad` marks a leaf you want a gradient
for; module parameters are marked already.

#### `GradientsParams` and the optimizer step

Connecting raw gradients back to named parameters.

```
use burn::nn::{Linear, LinearConfig};
use burn::optim::{GradientsParams, SgdConfig};
use burn::prelude::*;

#[derive(Module, Debug)]
struct Tiny {
    layer: Linear,
}

let device = Device::default().autodiff();
let mut model = Tiny { layer: LinearConfig::new(2, 1).init(&device) };
let mut optimizer = SgdConfig::new().init();

let before: Vec<f32> = model.layer.weight.val().into_data().try_to_vec().unwrap();

let loss = model.layer.forward(Tensor::<2>::ones([1, 2], &device)).sum();
let gradients = GradientsParams::from_grads(loss.backward(), &model);
model = optimizer.step(0.1, model, gradients);

let after: Vec<f32> = model.layer.weight.val().into_data().try_to_vec().unwrap();
assert_ne!(before, after); // the step moved the parameters
```

**When to use it:** in any hand-written loop. `backward()` produces gradients
keyed by tensor identity, and `GradientsParams::from_grads` re-keys them by
module path so the optimizer knows which parameter each belongs to — skipping
that step is the usual reason a custom loop silently trains nothing.
`AdamConfig`, `AdamWConfig`, `SgdConfig` and `RmsPropConfig` all produce an
optimizer with the same `step` signature.

#### `SupervisedTraining`

The managed loop: epochs, validation, metrics, checkpoints and a terminal
dashboard.

```
// no-compile — needs a Dataset and Batcher; see the Burn Book for a full example
use burn::train::metric::LossMetric;
use burn::train::{Learner, SupervisedTraining};

let training = SupervisedTraining::new(artifact_dir, train_loader, valid_loader)
    .metric_train_numeric(LossMetric::new())
    .metric_valid_numeric(LossMetric::new())
    .num_epochs(10)
    .with_default_checkpointers()
    .summary();

// The Learner holds what changes; the training holds how it is run.
let result = training.launch(Learner::new(model, optimizer, learning_rate));
let trained = result.model;
```

**When to use it:** for ordinary supervised training, where writing the loop by
hand buys nothing. It wants your model to implement `TrainStep` and
`InferenceStep`, and your data to arrive through a `Batcher` and `DataLoader`,
which is a good deal more setup than the loop above — the payoff is
checkpointing, early stopping, metric tracking and the TUI for free. Note that
this replaced the `LearnerBuilder` that earlier versions built training with, so
published examples will not match. The `train` feature is required, and `tui`
adds the dashboard.

### Storage

#### `save_into` and `load_from`

Weights to a file and back, with a report.

```
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;
use burn::store::{BurnpackStore, ModuleSnapshot};

#[derive(Module, Debug)]
struct Small {
    layer: Linear,
}

let device = Device::default();
let model = Small { layer: LinearConfig::new(3, 3).init(&device) };

let path = std::env::temp_dir().join("burn-api-map-example.bpk");
let mut store = BurnpackStore::from_file(&path);
model.save_into(&mut store).unwrap();

let mut target = Small { layer: LinearConfig::new(3, 3).init(&device) };
let mut reader = BurnpackStore::from_file(&path);
let result = target.load_from(&mut reader).unwrap();

assert_eq!(result.applied.len(), 2);
assert!(result.errors.is_empty());
```

**When to use it:** checkpointing, shipping a trained model, and importing one
trained elsewhere. Always look at the `ApplyResult` — `applied`, `skipped`,
`unused` and `errors` — rather than only checking the `Result`: a model whose
field names drifted will load "successfully" with nothing applied.
`SafetensorsStore` and `PytorchStore` cover those formats, and `KeyRemapper`
exists for when the names do not line up, which with a PyTorch checkpoint they
usually do not.
