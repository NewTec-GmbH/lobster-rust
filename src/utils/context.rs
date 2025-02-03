//! Context module to efficiently track namespaces thoruhg module and file nestings.

// BSD 3-Clause License
//
// Copyright (c) 2025, NewTec GmbH
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions
//    and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of
//    conditions and the following disclaimer in the documentation and/or other materials provided
//    with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to
//    endorse or promote products derived from this software without specific prior written
//    permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICU5LAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::{iter::Sum, ops};

/// Context struct to handle nested namespaces.
///
/// The struct allows efficient creation, representation and most importantly combination of
/// contexts. With contexts it's easy to represent nested modules and directories.
/// This allows the easy creation of names and tags for traceable nodes.
#[derive(Debug, Clone)]
pub(crate) enum Context {
    Empty,
    Stacked(Vec<String>),
}

impl Context {
    /// Create a new Context from a &str
    ///
    /// The function expects the namespaces in the str to be separated by '.'.
    ///
    /// ### Parameters
    /// * `source` - &str source for the new Context.
    ///
    /// ### Returns
    /// New Context.
    pub(crate) fn from_str(source: &str) -> Self {
        if source.is_empty() {
            Context::Empty
        } else {
            let stack = source.split('.').map(|s| s.to_string()).collect();
            Context::Stacked(stack)
        }
    }

    /// Create a String representation of the Context.
    ///
    /// The function will separate the namespaces in the representation by '.'.
    ///
    /// ### Returns
    /// String representation of the Context.
    pub(crate) fn to_str(&self) -> String {
        match self {
            Context::Empty => "".to_string(),
            Context::Stacked(stack) => stack.join("."),
        }
    }

    /// Combine with another context into a new Context
    ///
    /// This will create a new context with the other Context nested in this Context.
    ///
    /// ### Parameters
    /// * `other` - Other Context to combine with.
    ///
    /// ### Returns
    /// New Context that is a combination of both.
    pub(crate) fn combine(&self, other: &Self) -> Self {
        match (self, other) {
            (Context::Empty, Context::Empty) => Context::Empty,
            (Context::Stacked(s), Context::Empty) => Context::Stacked(s.clone()),
            (Context::Empty, Context::Stacked(s)) => Context::Stacked(s.clone()),
            (Context::Stacked(s1), Context::Stacked(s2)) => {
                let mut new_stack = s1.clone();
                new_stack.extend(s2.clone());
                Context::Stacked(new_stack)
            }
        }
    }
}

impl ops::Add<Context> for Context {
    type Output = Context;

    fn add(self, rhs: Context) -> Self::Output {
        self.combine(&rhs)
    }
}

impl ops::Add<&Context> for &Context {
    type Output = Context;

    fn add(self, rhs: &Context) -> Self::Output {
        self.combine(rhs)
    }
}

impl ops::Add<Context> for &Context {
    type Output = Context;

    fn add(self, rhs: Context) -> Self::Output {
        self.combine(&rhs)
    }
}

impl ops::Add<String> for &Context {
    type Output = Context;

    fn add(self, rhs: String) -> Self::Output {
        self.combine(&Context::from_str(&rhs))
    }
}

impl<'a> Sum<&'a Context> for Context {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Context::Empty, |acc, c| &acc + c)
    }
}
