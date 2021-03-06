use subject::*;
use std::sync::Mutex;
use observable::Observable;
use std::sync::Arc;
use unsub_ref::UnsubRef;
use observable::Observer;
use std::any::Any;

pub struct BehaviorSubject<V:Clone+'static>
{
    v: Mutex<Option<V>>,
    subj: Subject<V>
}

impl<V:Clone+'static> BehaviorSubject<V>
{
    pub fn new(value: Option<V>) -> BehaviorSubject<V>
    {
        BehaviorSubject{ subj: Subject::new(), v: Mutex::new(value)}
    }

    pub fn value(&self) -> Option<V>
    {
        let guard = self.v.lock().unwrap();
        guard.as_ref().map(|v| v.clone())
    }

    pub fn clear_value(&self)
    {
        let mut guard = self.v.lock().unwrap();
        *guard = None;
    }
}

impl<V:Clone+'static> Observable<V> for BehaviorSubject<V>
{
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync>) -> UnsubRef<'static>
    {
        if dest._is_closed() {
            return UnsubRef::empty();
        }

        {
            let guard = self.v.lock().unwrap();
            if let Some(ref val) = *guard { dest.next(val.clone()); }
        }

        if dest._is_closed() || self._is_closed() {
            return UnsubRef::empty();
        }

        self.subj.sub(dest)
    }
}

impl<V:Clone+'static> Observer<V> for BehaviorSubject<V>
{
    fn next(&self, v: V)
    {
        if self._is_closed() { return; }

        {
            let mut guard = self.v.lock().unwrap();
            *guard = Some(v.clone());
        }

        self.subj.next(v);
    }

    fn err(&self, e: Arc<Any+Send+Sync>)
    {
        self.clear_value();
        self.subj.err(e);
    }

    fn complete(&self)
    {
        self.subj.complete();
    }

    fn _is_closed(&self) -> bool { self.subj._is_closed() }
}


#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn value()
    {
        let s = BehaviorSubject::<i32>::new(None);
        assert!(s.value().is_none());

        s.next(1);
        assert_eq!(s.value().unwrap(), 1);

        s.err(Arc::new("error"));
        s.next(2);
        assert!(s.value().is_none());
    }
}