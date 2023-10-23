use crate::platform::agent::base::AgentInfoDescription;

pub mod ams;
pub mod mts;

pub(crate) trait Service<T> {
    fn get_aid(&self) -> &AgentInfoDescription;
    fn get_name(&self) -> &str;
}

pub(crate) trait Directory<D, I, O> {
    //manage elements
    fn add_element(&mut self, key: I, value: O);
    fn get_element(&self, element: I) -> Option<O>;
    fn remove_element(&mut self, element: I);
    //manage directory
    fn get_directory(&self) -> &D;
    fn clear_directory(&mut self);
    fn refresh_directory(&mut self) {}
}
