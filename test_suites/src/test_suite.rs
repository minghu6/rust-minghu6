use std::{collections::HashMap, fmt::Debug};

use log::{debug, trace};
use serde::Deserialize;


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Validate {
    fn validate(&self);
}

pub trait TestSuite {
    type EG: Validate;
    type CG;
    type IU: Debug;

    fn new_test_context(&mut self, name: &str) -> TestContext<Self::EG, Self::CG, Self::IU>;

    fn interpret(
        ctx: &mut TestContext<Self::EG, Self::CG, Self::IU>,
    );

    fn run_case(&mut self, case: TestCase<Self::IU>) {
        let TestCase { name, data } = case;
        let mut ctx = self.new_test_context(&name);

        for unit in data {
            trace!("{unit:?}");
            ctx.push(unit);
            Self::interpret(&mut ctx);
        }

        debug!("pass {name}");
    }

    fn load_fixeddata() -> TestDataTable<Self::IU>;

    fn test_fixeddata(&mut self) {
        let test_data = Self::load_fixeddata();

        for case in test_data.fixeddata {
            self.run_case(case);
        }
    }

    fn new_random_input(&mut self, len: usize, upper_bound: usize) -> Vec<Vec<Self::IU>>;

    fn test_randomdata(&mut self, len: usize, upper_bound: usize) {
        debug!("[Random Test]");

        let mut length_counter = HashMap::<usize, usize>::new();

        for case_data in self.new_random_input(len, upper_bound) {
            let len = case_data.len();
            let len_id = *length_counter.entry(len).or_insert(0);

            let case_name = format!("random-{len}-{len_id}");

            let case = TestCase {
                name: case_name,
                data: case_data,
            };

            self.run_case(case);
        }
    }

}

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Deserialize, Debug)]
pub struct TestDataTable<T> {
    fixeddata: Vec<TestCase<T>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TestCase<T> {
    name: String,
    data: Vec<T>,
}

pub struct TestContext<EG, CG, IU> {
    pub(crate) eg: EG,
    pub(crate) cg: CG,
    pub(crate) name: String,
    pub(crate) input: Vec<IU>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<EG, CG, IU> TestContext<EG, CG, IU> {
    pub(crate) fn push(&mut self, value: IU) {
        self.input.push(value);
    }

    pub(crate) fn cur(&self) -> Option<&IU> {
        self.input.last()
    }
}
