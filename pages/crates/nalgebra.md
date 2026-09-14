---
title: "nalgebra"
version: "0.35.0"
publisher: "Sébastien Crozet (sebcrozet), Emilia Bopp (milibopp)"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-14"
summary: "Linear algebra with dimensions in the type system: vectors, matrices, transformations and decompositions, where multiplying a 3×2 by a 4×4 is a compile error rather than a runtime one."
categories: ["mathematics", "graphics", "no-std"]
repository: "https://github.com/dimforge/nalgebra"
docs: "https://www.nalgebra.rs/docs"
---

## Overview

Anything with geometry in it needs linear algebra: a renderer composing
transforms, a robot arm solving for joint angles, a simulation integrating
forces, a fit over measured data. `nalgebra` provides the vectors, matrices and
transformations for that, with the dimensions carried in the types:

```
use nalgebra::{Matrix3, Vector3};

let rotation: Matrix3<f64> = Matrix3::new(
    0.0, -1.0, 0.0,
    1.0,  0.0, 0.0,
    0.0,  0.0, 1.0,
);
let point = Vector3::new(1.0, 0.0, 0.0);

let turned = rotation * point;

assert!((turned.x - 0.0).abs() < 1e-10);
assert!((turned.y - 1.0).abs() < 1e-10);

// Multiplying mismatched dimensions does not compile at all.
```

**The dimensions are the selling point.** A `Matrix3x2` times a `Vector4` is
rejected by the compiler, not by an assertion at run time, which removes a whole
category of bug from code that is otherwise easy to get subtly wrong. Sizes can
also be dynamic — `DMatrix` and `DVector` decide at run time — and the two mix,
so a fixed-size transform can act on a dynamically sized set of points.

**The cost is the type signatures.** `Matrix<T, R, C, S>` is generic over scalar,
rows, columns and storage, so an error message mentions `ArrayStorage`,
`Const<3>` and `ShapeConstraint` before it mentions your mistake. This is the
main complaint about the crate and it is a fair one: the aliases (`Vector3`,
`Matrix4`, `Isometry3`) are what you write, and reading the full type is
something you learn to skip.

**Two distinctions repay learning early.**

A **point** is a position; a **vector** is a displacement. `Point3` and
`Vector3` are different types because translating a position moves it and
translating a direction does not. Graphics code that conflates them produces
normals that get dragged around by the camera.

A **transformation type** carries what it guarantees. `Rotation3` is a rotation
and nothing else, `Isometry3` is rotation plus translation and so preserves
distances, `Similarity3` adds uniform scaling. Using a typed transform rather
than a bare `Matrix4` means its inverse is cheap and exact instead of a general
matrix inversion that might be singular.

**Alternatives worth weighing.** `glam` is faster for games and far simpler —
2D/3D only, no dimension generics, SIMD by default — and is the right default
for a renderer or a game. `ndarray` targets n-dimensional numeric arrays for
data work rather than geometry. `nalgebra` is the one to choose when you need
decompositions, statically-checked dimensions beyond 4, or the typed
transformation hierarchy; it is the foundation of the `rapier` physics engines.

It requires Rust 1.89, and works `no_std` for the fixed-size parts, since
dynamic matrices need an allocator.

## When to use it

### Use case: Composing a camera transform

Transform types compose with `*`, and the result type says what the composition
guarantees.

```
use nalgebra::{Isometry3, Point3, Translation3, UnitQuaternion, Vector3};

// A camera 5 units back, rotated a quarter turn about Y.
let position = Translation3::new(0.0, 0.0, 5.0);
let facing = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f64::consts::FRAC_PI_2);
let camera: Isometry3<f64> = Isometry3::from_parts(position, facing);

// The view transform is the inverse — exact, not a matrix inversion.
let view = camera.inverse();
let world_point = Point3::new(0.0, 0.0, 0.0);
let in_view = view * world_point;

assert!((in_view.z - 0.0).abs() < 1e-10);
assert!((in_view.x - 5.0).abs() < 1e-10);

// Composition stays an Isometry3, so it still has a cheap inverse.
let combined = camera * Isometry3::translation(1.0, 0.0, 0.0);
assert!((combined.inverse() * (combined * world_point) - world_point).norm() < 1e-10);
```

**Why it fits:** `Isometry3::inverse` is a transpose and a negation because the
type guarantees no scaling or shear. Storing the same transform as a `Matrix4`
would mean a general inverse — slower, numerically worse, and fallible.

### Use case: Solving a linear system

The classic numerical task: `Ax = b` for an unknown `x`. A decomposition does it
without ever forming the inverse.

```
use nalgebra::{Matrix3, Vector3};

// 2x +  y -  z =  8
// -3x - y + 2z = -11
// -2x + y + 2z = -3
let a = Matrix3::new(
     2.0,  1.0, -1.0,
    -3.0, -1.0,  2.0,
    -2.0,  1.0,  2.0,
);
let b = Vector3::new(8.0, -11.0, -3.0);

let x = a.lu().solve(&b).expect("matrix is singular");

assert!((x - Vector3::new(2.0, 3.0, -1.0)).norm() < 1e-10);

// Check by substitution.
assert!((a * x - b).norm() < 1e-10);
```

**Why it fits:** `solve` returns `Option`, so a singular matrix is a value you
handle rather than a silent `NaN`. Computing `a.try_inverse()` and multiplying
would be slower and less accurate — forming the inverse is almost never what you
actually want.

### Use case: Least squares over measured data

Fitting a line to noisy samples, with dimensions known only at run time.

```
use nalgebra::{DMatrix, DVector};

// Fit y = m*x + c through three points.
let xs: [f64; 3] = [0.0, 1.0, 2.0];
let ys: [f64; 3] = [1.0, 3.0, 5.0];

// Design matrix: one row per sample, columns [x, 1].
let a = DMatrix::from_fn(xs.len(), 2, |r, c| if c == 0 { xs[r] } else { 1.0 });
let b = DVector::from_row_slice(&ys);

let solution = a.clone().svd(true, true).solve(&b, 1e-12).unwrap();

assert!((solution[0] - 2.0).abs() < 1e-9); // slope
assert!((solution[1] - 1.0).abs() < 1e-9); // intercept
```

**Why it fits:** the sample count is not known until run time, so the rows are
`Dyn` while the two columns stay fixed. SVD handles the over-determined system
and is the numerically stable way to do it — normal equations are shorter to
write and lose precision on ill-conditioned data.

## API map

The crate is large, so this is a curated selection: the types you construct, the
operations you reach for, and the transformations and decompositions that are
the reason to choose it over a simpler vector library.

### Vectors and points

#### `Vector3` and friends

Fixed-size column vectors, from `Vector1` to `Vector6`.

```
use nalgebra::{Vector2, Vector3};

let v: Vector3<f64> = Vector3::new(1.0, 2.0, 2.0);

assert_eq!(v.x, 1.0);
assert_eq!(v.norm(), 3.0); // <- sqrt(1 + 4 + 4)
assert!((v.normalize().norm() - 1.0).abs() < 1e-10);

// Component-wise arithmetic, and scalar multiplication.
let sum: Vector2<f64> = Vector2::new(1.0, 2.0) + Vector2::new(3.0, 4.0);
assert_eq!(sum, Vector2::new(4.0, 6.0));
assert_eq!(Vector2::<f64>::new(1.0, 2.0) * 2.0, Vector2::new(2.0, 4.0));
```

**When to use it:** directions, velocities, forces, offsets — anything that is a
displacement rather than a location. `x`, `y`, `z` and `w` are field accessors,
and indexing works too, which matters in generic code where the dimension is a
parameter.

#### `Point3`

A position, distinguished from a vector by the type.

```
use nalgebra::{Point3, Vector3};

let origin = Point3::new(0.0, 0.0, 0.0);
let offset = Vector3::new(1.0, 2.0, 3.0);

// Point + Vector is a Point; Point - Point is a Vector.
let moved: Point3<f64> = origin + offset;
let back: Vector3<f64> = moved - origin;

assert_eq!(moved, Point3::new(1.0, 2.0, 3.0));
assert_eq!(back, offset);
```

**When to use it:** vertices, positions, anything with a location. The
arithmetic in the example is the whole reason the type exists — the compiler
enforces that adding two positions is meaningless, and that a translation
applies to a point but not to a direction.

#### `dot` and `cross`

The two products, with their usual geometric meanings.

```
use nalgebra::Vector3;

let a: Vector3<f64> = Vector3::new(1.0, 0.0, 0.0);
let b: Vector3<f64> = Vector3::new(0.0, 1.0, 0.0);

assert_eq!(a.dot(&b), 0.0); // <- perpendicular
assert_eq!(a.cross(&b), Vector3::new(0.0, 0.0, 1.0)); // <- right-handed

// dot is also how you project and measure angle.
let along: Vector3<f64> = Vector3::new(2.0, 2.0, 0.0);
assert!((along.dot(&a) - 2.0).abs() < 1e-10);
```

**When to use it:** `dot` for projection, angle, and "is this facing away";
`cross` for surface normals and torque. `cross` is 3D only — the type system
enforces that, since it is not defined for other dimensions.

### Matrices

#### `Matrix3::new` and the `matrix!` macro

Fixed-size matrices, written row by row.

```
use nalgebra::{matrix, Matrix2, Matrix3};

let m = Matrix3::new(
    1.0, 2.0, 3.0,
    4.0, 5.0, 6.0,
    7.0, 8.0, 9.0,
);
assert_eq!(m[(0, 2)], 3.0); // <- (row, column)

// The macro reads closer to written maths.
let n = matrix![1.0, 2.0;
                3.0, 4.0];
assert_eq!(n, Matrix2::new(1.0, 2.0, 3.0, 4.0));
```

**When to use it:** any fixed-size matrix. Note that `new` takes arguments in
**row-major** order for readability even though storage is column-major — which
matters when you hand the data to a graphics API expecting columns, where
`as_slice` gives you the storage order rather than the written one.

#### `DMatrix`

Dimensions decided at run time.

```
use nalgebra::DMatrix;

let m = DMatrix::from_row_slice(2, 3, &[
    1.0, 2.0, 3.0,
    4.0, 5.0, 6.0,
]);

assert_eq!(m.nrows(), 2);
assert_eq!(m.ncols(), 3);
assert_eq!(m[(1, 0)], 4.0);

// Dimension errors are now runtime panics rather than compile errors.
let product = &m * m.transpose();
assert_eq!(product.shape(), (2, 2));
```

**When to use it:** data whose size comes from a file, a sensor or user input.
The trade is explicit — you gain flexibility and lose the compile-time
dimension check, so mismatches become panics. `from_fn`, `from_row_slice` and
`from_element` are the usual constructors.

#### Transpose, determinant and inverse

The standard operations, with fallibility where it belongs.

```
use nalgebra::Matrix2;

let m: Matrix2<f64> = Matrix2::new(4.0, 7.0, 2.0, 6.0);

assert_eq!(m.transpose(), Matrix2::new(4.0, 2.0, 7.0, 6.0));
assert!((m.determinant() - 10.0).abs() < 1e-10);

let inv = m.try_inverse().unwrap();
assert!((m * inv - Matrix2::identity()).norm() < 1e-10);

// A singular matrix returns None rather than infinities.
assert!(Matrix2::<f64>::new(1.0, 2.0, 2.0, 4.0).try_inverse().is_none());
```

**When to use it:** `transpose` freely; `try_inverse` rarely. If you are
inverting in order to solve a system, use a decomposition instead — it is faster
and more accurate. Inversion is for when you genuinely need the inverse matrix
itself, such as a normal matrix in shading.

#### Slicing and views

Borrowing part of a matrix without copying it.

```
use nalgebra::{Matrix3, Vector3};

let m = Matrix3::new(
    1.0, 2.0, 3.0,
    4.0, 5.0, 6.0,
    7.0, 8.0, 9.0,
);

assert_eq!(m.column(0), Vector3::new(1.0, 4.0, 7.0));
assert_eq!(m.row(1)[2], 6.0);

// A fixed-size view into the top-left corner.
let corner = m.fixed_view::<2, 2>(0, 0);
assert_eq!(corner[(1, 1)], 5.0);
```

**When to use it:** operating on a submatrix — a rotation block inside a 4×4, a
column of observations. Views borrow, so there is no copy, and `fixed_view`
keeps the dimensions static so the result still type-checks.

### Transformations

#### `Rotation3` and `UnitQuaternion`

Two representations of a rotation, both guaranteeing they *are* rotations.

```
use nalgebra::{Rotation3, UnitQuaternion, Vector3};

let angle = std::f64::consts::FRAC_PI_2;

let r = Rotation3::from_axis_angle(&Vector3::z_axis(), angle);
let q = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), angle);

let v = Vector3::new(1.0, 0.0, 0.0);
assert!((r * v - q * v).norm() < 1e-10); // <- same rotation

// Inverting is free for both: transpose, or conjugate.
assert!((r.inverse() * (r * v) - v).norm() < 1e-10);
```

**When to use it:** `UnitQuaternion` for anything you interpolate or accumulate
— it has no gimbal lock and renormalises cheaply, which is why animation and
physics use it. `Rotation3` when you want the matrix itself, since applying it
is a plain multiply.

#### `Isometry3`

Rotation plus translation: the transform of a rigid body.

```
use nalgebra::{Isometry3, Point3, Vector3};

let placement = Isometry3::new(Vector3::new(1.0, 0.0, 0.0), Vector3::z() * std::f64::consts::PI);

let p = Point3::new(1.0, 0.0, 0.0);
let moved = placement * p;

// A half turn about z, then a shift along x.
assert!((moved.x - 0.0).abs() < 1e-10);

// Distances are preserved, which is what "isometry" means.
let q = Point3::new(2.0, 0.0, 0.0);
assert!(((placement * q) - moved).norm() - (q - p).norm() < 1e-10);
```

**When to use it:** the pose of anything rigid — a camera, a robot link, a
physics body. It is the type to reach for by default in 3D, because it says what
it preserves and composes without accumulating shear the way repeated `Matrix4`
multiplication can.

#### `Perspective3` and homogeneous coordinates

Projection, which is where the 4×4 matrix becomes necessary.

```
use nalgebra::{Perspective3, Point3};

let projection = Perspective3::new(16.0 / 9.0, std::f64::consts::FRAC_PI_4, 0.1, 100.0);

let point = Point3::new(0.0, 0.0, -5.0); // <- in front of the camera
let projected = projection.project_point(&point);

assert!(projected.x.abs() < 1e-10);
assert!(projected.z > -1.0 && projected.z < 1.0); // <- inside the clip volume

// The raw matrix, for handing to a graphics API.
let m = projection.as_matrix();
assert_eq!(m.shape(), (4, 4));
```

**When to use it:** building a render pipeline's projection stage. A perspective
divide is not an affine transform, which is why it lives in a 4×4 and why this
type exists separately from `Isometry3` — it does not preserve distances or
parallelism.

### Decompositions

#### `lu` and `solve`

LU decomposition: the general-purpose way to solve a square system.

```
use nalgebra::{Matrix3, Vector3};

let a = Matrix3::new(
    4.0, 3.0, 0.0,
    3.0, 4.0, -1.0,
    0.0, -1.0, 4.0,
);
let b = Vector3::new(24.0, 30.0, -24.0);

let lu = a.lu();
let x = lu.solve(&b).unwrap();

assert!((a * x - b).norm() < 1e-10);

// The decomposition is reusable for more right-hand sides.
let x2 = lu.solve(&Vector3::new(1.0, 0.0, 0.0)).unwrap();
assert!((a * x2 - Vector3::new(1.0, 0.0, 0.0)).norm() < 1e-10);
```

**When to use it:** any square system with no special structure. Decompose once
and solve repeatedly — that is the saving over `try_inverse`, and the reason
`lu()` returns a value rather than doing it all in one call.

#### `cholesky`

The fast path for symmetric positive-definite matrices.

```
use nalgebra::{Matrix3, Vector3};

// Symmetric, positive-definite.
let a = Matrix3::new(
    4.0, 1.0, 1.0,
    1.0, 3.0, 0.0,
    1.0, 0.0, 2.0,
);

let chol = a.cholesky().expect("not positive-definite");
let x = chol.solve(&Vector3::new(1.0, 2.0, 3.0));

assert!((a * x - Vector3::new(1.0, 2.0, 3.0)).norm() < 1e-10);

// It fails rather than producing nonsense on an unsuitable matrix.
assert!(Matrix3::new(0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0).cholesky().is_none());
```

**When to use it:** covariance matrices, least-squares normal equations, physics
solvers — anywhere the matrix is symmetric positive-definite. It is roughly
twice as fast as LU, and the `None` doubles as a check that your matrix really
has the property you assumed.

#### `svd`

The most informative and most expensive decomposition.

```
use nalgebra::Matrix2;

let m: Matrix2<f64> = Matrix2::new(3.0, 0.0, 0.0, 2.0);
let svd = m.svd(true, true);

// Singular values, largest first.
assert!((svd.singular_values[0] - 3.0).abs() < 1e-10);
assert!((svd.singular_values[1] - 2.0).abs() < 1e-10);

// Rank and conditioning fall out of them.
assert_eq!(svd.rank(1e-10), 2);
```

**When to use it:** least squares, rank, pseudo-inverse, and diagnosing an
ill-conditioned system. It is the slowest decomposition here and the one that
never fails, so it is the right answer when the matrix might be singular or
non-square and you need an answer anyway.

#### `symmetric_eigen`

Eigenvalues and eigenvectors, for symmetric matrices.

```
use nalgebra::Matrix2;

let m = Matrix2::new(2.0, 1.0, 1.0, 2.0);
let eigen = m.symmetric_eigen();

let mut values: Vec<f64> = eigen.eigenvalues.iter().copied().collect();
values.sort_by(|a, b| a.partial_cmp(b).unwrap());

assert!((values[0] - 1.0).abs() < 1e-10);
assert!((values[1] - 3.0).abs() < 1e-10);
```

**When to use it:** principal component analysis, vibration modes, inertia
tensors — the cases where the matrix is symmetric by construction. The symmetric
algorithm is faster than the general one and returns real eigenvalues, which the
general case cannot promise.
