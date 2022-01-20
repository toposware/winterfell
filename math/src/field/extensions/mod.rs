// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

mod quadratic;
pub use quadratic::QuadExtension;

mod cubic;
pub use cubic::CubeExtension;

use super::{ExtensibleField, FieldElement};
