/*
MIT License
Copyright (c) 2021 Germán Molina
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use model::{Model,SimulationState, SimulationStateHeader};
use weather::WeatherTrait;
use calendar::Date;
use communication::{MetaOptions, SimulationModel, ErrorHandling};
use std::borrow::Borrow;

struct ModelA {}
impl ErrorHandling for ModelA{
    fn module_name()->&'static str {
        "A"
    }    
}
impl SimulationModel for ModelA{
    type OutputType = Self;
    type OptionType = OptionsA;
    type AllocType = ();
    fn new<M: Borrow<Model>>(_meta: &MetaOptions, _options: Self::OptionType, _model : M, _state: &mut SimulationStateHeader, _n: usize)->Result<Self::OutputType,String>{
        todo!()
    }
    fn march<W: WeatherTrait, M: Borrow<Model>>(&self, _date: Date, _weather: &W, _model: M, _state: &mut SimulationState, _alloc: &mut () )->Result<(),String>{
        todo!()
    }

    fn allocate_memory(&self)->Result<Self::AllocType, String>{
        Ok(())
    }


}

struct OptionsA {
    _n: usize,
}





struct ModelB {}
impl ErrorHandling for ModelB{
    fn module_name()->&'static str {
        "A"
    }    
}
impl SimulationModel for ModelB{
    type OutputType = Self;
    type OptionType = OptionsB;
    type AllocType = ();

    fn new<M: Borrow<Model>>(_meta: &MetaOptions, _options: Self::OptionType, _model : M, _state: &mut SimulationStateHeader, _n: usize)->Result<Self::OutputType,String>{
        todo!()
    }
    fn march<W: WeatherTrait, M: Borrow<Model>>(&self, _date: Date, _weather: &W, _model: M, _state: &mut SimulationState, _alloc: &mut ())->Result<(),String>{
        todo!()
    }

    fn allocate_memory(&self)->Result<Self::AllocType, String>{
        Ok(())
    }
}

struct OptionsB {
    _n: usize,
}

#[allow(dead_code)]
struct MultiModel{
    a: ModelA,
    b: ModelB,
}


#[test]
fn test_compile(){

}