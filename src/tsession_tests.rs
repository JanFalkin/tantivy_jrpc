use crate::{Serialize, Deserialize};
use crate::InternalCallResult;
use crate::json;
use crate::info;
use crate::tantivy_jpc;

pub mod tests {
    extern crate tempdir;

    use tempdir::TempDir;
    use uuid::{Uuid};

    use super::*;
    use serde_json::Map;


    pub static mut GSIZE:usize = 0;

    macro_rules! call_simple_type {
        //() => {};
        ($self:ident, $j_param:ident, $method:literal) => {
            {
                let v = &$self.call_jpc("builder".to_string(), $method.to_string(), $j_param, true);
                let temp_map:serde_json::Value = match serde_json::from_slice(v){
                    Ok(sv) => sv,
                    Err(e) => {
                        info!("return value not json {e}");
                        return -22
                    },
                };
                temp_map["field"].as_i64().unwrap_or(0)
            }
        }
     }




    #[derive(Debug)]
    pub struct FakeContext<'a>{
        pub id:String,
        pub buf:Vec<u8>,
        pub ret_len:&'a mut usize,
        pub dirs:&'a mut Vec<TempDir>,

    }
    #[derive(Debug)]
    pub struct TestDocument<'a>{
        pub     temp_dir:String,
        pub ctx:    &'a FakeContext<'a>,

    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct TestDocResult {
        pub opstamp: u64,
    }
    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct TestCreateDocumentResult{
        pub document_count: usize
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct TestBuilderAddTextResult {
        pub schema: serde_json::Value,
    }
    #[derive(Serialize, Deserialize, Clone, Debug, Default)]
    pub struct TestTitleResult {
        pub title: Vec<String>,
    }


    pub struct TestIndex<'a>{
        ctx:    &'a FakeContext<'a>,
        temp_dir: String,
    }

    pub struct TestIndexReader<'a>{
        ctx:    &'a FakeContext<'a>,
    }

    pub struct TestQueryParser<'a>{
        ctx:    &'a FakeContext<'a>,
    }

    pub struct TestSearcher<'a>{
        ctx:    &'a FakeContext<'a>,
    }

    impl<'a> TestSearcher<'a>{
        pub fn search(&mut self)-> InternalCallResult<String>{
            let b = self.ctx.call_jpc("searcher".to_string(), "search".to_string(), json!({}),true);
            let s = std::str::from_utf8(&b).unwrap();
            Ok(s.to_string())
        }
        pub fn fuzzy_search(&mut self)-> InternalCallResult<String>{
            let b = self.ctx.call_jpc("fuzzy_searcher".to_string(), "fuzzy_searcher".to_string(), json!({}),true);
            let s = std::str::from_utf8(&b).unwrap();
            Ok(s.to_string())
        }
    }

    impl<'a> TestQueryParser<'a>{
        pub fn for_index(&mut self, v:Vec<String>)-> InternalCallResult<i32>{
            self.ctx.call_jpc("query_parser".to_string(), "for_index".to_string(), json!({
                "fields": v,
            }), false);
            Ok(0)
        }
        pub fn parse_query(&mut self, query:String) ->  InternalCallResult<TestSearcher> {
            self.ctx.call_jpc("query_parser".to_string(), "parse_query".to_string(), json!({"query": query}), false);
            Ok(TestSearcher{ctx:self.ctx})
        }
        pub fn parse_fuzzy_query(&mut self, term:String, field:String) ->  InternalCallResult<TestSearcher> {
            self.ctx.call_jpc("query_parser".to_string(), "parse_fuzzy_query".to_string(), json!({"term": [term], "field" : [field]}), false);
            Ok(TestSearcher{ctx:self.ctx})
        }

    }
    impl<'a> TestIndexReader<'a>{
        pub fn searcher(&mut self) -> InternalCallResult<TestQueryParser>{
            self.ctx.call_jpc("index_reader".to_string(), "searcher".to_string(), json!({}),false);
            Ok(TestQueryParser{ctx:self.ctx})
        }
    }

    impl<'a> TestIndex<'a>{
        pub fn add_document(&mut self, doc_id:i32) ->Result<u64, u32>{
            let _ = self.temp_dir;
            let s = self.ctx.call_jpc("indexwriter".to_string(), "add_document".to_string(), json!({"id": doc_id}), true);
            let resmap:TestDocResult = serde_json::from_slice(&s).unwrap();
            Ok(resmap.opstamp)
        }

        pub fn commit(&mut self) -> Result<i64, u32>{
            let r = self.ctx.call_jpc("indexwriter".to_string(), "commit".to_string(), json!({}), true);
            let i:Map<String,serde_json::Value> = serde_json::from_slice(&r).unwrap();
            Ok(i["id"].as_i64().unwrap())

        }
        pub fn reader_builder(&mut self)-> InternalCallResult<TestIndexReader>{
            self.ctx.call_jpc("index".to_string(), "reader_builder".to_string(), json!({}),false);
            Ok(TestIndexReader{ctx:self.ctx})
        }
    }

    impl<'a> TestDocument<'a>{
        pub fn create(&mut self) -> Result<usize, i32>{
            let tdc:TestCreateDocumentResult = serde_json::from_slice(&self.ctx.call_jpc("document".to_string(), "create".to_string(), json!({}), true)).unwrap();
            Ok(tdc.document_count)
        }
        pub fn add_text(&mut self, field:i32, value:String, doc_id:u32) -> i64 {
            self.ctx.call_jpc("document".to_string(), "add_text".to_string(), json!({"field":  field,"value":  value, "id":  self.ctx.id,  "doc_id": doc_id}),false);
            0
        }
        pub fn create_index(&mut self) -> Result<TestIndex, std::io::Error>{
            self.ctx.call_jpc("index".to_string(), "create".to_string(), json!({"directory":  self.temp_dir}), false);
            Ok(TestIndex{
                ctx:self.ctx,
                temp_dir:self.temp_dir.clone(),
            })
        }
    }

    impl<'a> Default for FakeContext<'a> {
       fn default() -> Self {
            Self::new()
       }
    }

    impl<'a> Drop for FakeContext<'a>{
        fn drop(&mut self) {
            for t in 0..self.dirs.len(){
                let td = self.dirs[t].as_ref();
                let _ = std::fs::remove_dir_all(td);
            }
        }
    }

    impl<'a> FakeContext<'a> {
        pub fn new() -> FakeContext<'a>{
            FakeContext{
                id: Uuid::new_v4().to_string(),
                buf: vec![0; 5000000],
                ret_len:Box::leak(Box::<usize>::new(0)),
                dirs:Box::leak(Box::<Vec<TempDir>>::default()),
            }
        }
        pub fn call_jpc(&'a self, object:String, method:String, params:serde_json::Value, do_ret:bool)-> Vec<u8>{
            let my_ret_ptr = Box::leak(Box::new(0));

            let call_p = json!({
                "id":     self.id,
                "jpc":    "1.0",
                "obj":    object,
                "method": method,
                "params": params,
            });
            let mut sp = serde_json::to_vec(&call_p).unwrap_or(vec![]);
            let p = Box::leak(Box::<Vec<u8>>::new(vec![0; 5000000]));
            info!("calling tantivy_jpc json = {}", call_p);
            unsafe{
                tantivy_jpc(sp.as_mut_ptr(), sp.len(), p.as_mut_ptr(), my_ret_ptr);
            }
            let sl = p[0..*my_ret_ptr].to_vec();
            if do_ret{
                let v:serde_json::Value = serde_json::from_slice(&sl).unwrap_or(json!({"result" : "empty"}));
                info!("Val = {}", v);
                match std::str::from_utf8(&sl){
                    Ok(s) => info!("stringified = {}", s),
                    Err(err) => panic!("ERROR = {err} p = {p:?}")
                };
                sl
            }else{
                println!("NO RETURNED REQUESTED");
                vec![]
            }
        }
        pub fn add_text_field(&mut self, name:String, a_type:i32, stored:bool) -> i64{
            let j_param = json!({
                "name":   name,
                "type":   a_type,
                "stored": stored,
                "id":     self.id,
            });
            let s = &self.call_jpc("builder".to_string(), "add_text_field".to_string(), j_param, true);
            info!("builder ret  = {:?}", s);
            let i:serde_json::Value = serde_json::from_slice(s).unwrap();
            i["field"].as_i64().unwrap()
        }

        pub fn add_date_field(&mut self, name:String, a_type:i32, stored:bool) -> i64{
            let j_param = json!({
                "name":   name,
                "type":   a_type,
                "stored": stored,
                "id":     self.id,
            });
            call_simple_type!(self, j_param, "add_date_field")
        }
        pub fn add_i64_field(&mut self, name:String, a_type:i32, stored:bool) -> i64{
            let j_param = json!({
                "name":   name,
                "type":   a_type,
                "stored": stored,
                "id":     self.id,
            });
            call_simple_type!(self, j_param, "add_i64_field")
        }
        pub fn add_u64_field(&mut self, name:String, a_type:i32, stored:bool) -> i64{
            let j_param = json!({
                "name":   name,
                "type":   a_type,
                "stored": stored,
                "id":     self.id,
            });
            call_simple_type!(self, j_param, "add_u64_field")
        }
        pub fn add_f64_field(&mut self, name:String, a_type:i32, stored:bool) -> i64{
            let j_param = json!({
                "name":   name,
                "type":   a_type,
                "stored": stored,
                "id":     self.id,
            });
            call_simple_type!(self, j_param, "add_f64_field")
        }
        pub fn build(&mut self, in_memory: bool)  -> InternalCallResult<TestDocument> {
            if in_memory{
                let _s = self.call_jpc("builder".to_string(), "build".to_string(), json!({}), false);
                return Ok(
                    TestDocument{ctx:self, temp_dir: "".to_string()}
                )
            }
            let td = TempDir::new("TantivyBitcodeTest")?;
            self.dirs.append(vec![td].as_mut());
            let td_ref:&TempDir = self.dirs.last().unwrap();
            let s = self.call_jpc("builder".to_string(), "build".to_string(), json!({}), false);
            info!("build returned={:?}", s);
            Ok(TestDocument{
                ctx:self,
                temp_dir: td_ref.path().to_owned().to_str().unwrap().to_string(),
            })
        }
    }


    #[test]
    fn basic_index(){
        unsafe{crate::init()};
        let mut ctx = FakeContext::new();
        assert_eq!(ctx.add_text_field("title".to_string(), 2, true), 0);
        assert_eq!(ctx.add_text_field("body".to_string(), 2, true), 1);
        let mut td = match ctx.build(false){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("failed with error {}", e.to_string()));
            }
        };
        let doc1 = match td.create(){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("doc1 create failed error {}", e.to_string()));
            }
        };

        let doc2 = match td.create(){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("doc2 create failed error {}", e.to_string()));
            }
        };
        assert_eq!(td.add_text(0, "The Old Man and the Sea".to_string(), doc1 as u32), 0);
        assert_eq!(td.add_text(1, "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish.".to_string(), doc1 as u32), 0);
        assert_eq!(td.add_text(0, "Of Mice and Men".to_string(), doc2 as u32), 0);
        assert_eq!(td.add_text(1, r#"A few miles south of Soledad, the Salinas River drops in close to the hillside bank and runs deep and green. The water is warm too, for it has slipped twinkling over the yellow sands in the sunlight before reaching the narrow pool. On one side of the river the golden foothill slopes curve up to the strong and rocky Gabilan Mountains, but on the valley side the water is lined with trees—willows fresh and green with every spring, carrying in their lower leaf junctures the debris of the winter's flooding; and sycamores with mottled, white, recumbent limbs and branches that arch over the pool"#.to_string(), doc2 as u32), 0);
        let mut ti = match td.create_index(){
            Ok(i) => i,
            Err(e) => panic!("failed to create index err ={} ", e)
        };
        let op1 = ti.add_document(doc1 as i32).unwrap();
        let op2 = ti.add_document(doc2 as i32).unwrap();
        assert_eq!(op1, 0);
        assert_eq!(op2, 1);
        ti.commit().unwrap();
        let mut rb = ti.reader_builder().unwrap();
        let mut qp = rb.searcher().unwrap();
        qp.for_index(vec!["title".to_string()]).unwrap();
        let mut searcher = qp.parse_query("Sea".to_string()).unwrap();
        let sres = &searcher.search().unwrap();
        let title_result:TestTitleResult = serde_json::from_str(sres).unwrap();
        assert_eq!(title_result.title[0], "The Old Man and the Sea".to_string());
    }
    #[test]
    fn basic_index_fuzzy(){
        unsafe{crate::init()};
        let mut ctx = FakeContext::new();
        assert_eq!(ctx.add_text_field("title".to_string(), 2, true), 0);
        let mut td = match ctx.build(true){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("failed with error {}", e.to_string()));
            }
        };
        let doc1 = match td.create(){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("doc1 create failed error {}", e.to_string()));
            }
        };

        let doc2 = match td.create(){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("doc2 create failed error {}", e.to_string()));
            }
        };

        let doc3 = match td.create(){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("doc3 create failed error {}", e.to_string()));
            }
        };

        let doc4 = match td.create(){
            Ok(t) => t,
            Err(e) => {
                panic!("{}",format!("doc4 create failed error {}", e.to_string()));
            }
        };
        assert_eq!(td.add_text(0, "The Name of the Wind".to_string(), doc1 as u32), 0);
        assert_eq!(td.add_text(0, "The Diary of Muadib".to_string(), doc2 as u32), 0);
        assert_eq!(td.add_text(0, "A Dairy Cow".to_string(), doc3 as u32), 0);
        assert_eq!(td.add_text(0, "The Diary of a Young Girl".to_string(), doc4 as u32), 0);
        let mut ti = match td.create_index(){
            Ok(i) => i,
            Err(e) => panic!("failed to create index err ={} ", e)
        };
        let op1 = ti.add_document(doc1 as i32).unwrap();
        let op2 = ti.add_document(doc2 as i32).unwrap();
        let op3 = ti.add_document(doc3 as i32).unwrap();
        let op4 = ti.add_document(doc4 as i32).unwrap();
        assert_eq!(op1, 0);
        assert_eq!(op2, 1);
        assert_eq!(op3, 2);
        assert_eq!(op4, 3);
        ti.commit().unwrap();
        let mut rb = ti.reader_builder().unwrap();
        let mut qp = rb.searcher().unwrap();
        let mut searcher = qp.parse_fuzzy_query("Diary".to_string(), "title".to_string()).unwrap();
        let sres = &searcher.fuzzy_search().unwrap();
        let vret:Vec<serde_json::Value> = serde_json::from_str(sres).unwrap();
        assert_eq!(vret.len(), 2);
        println!("result = {vret:?}");
    }
    #[test]
    fn all_simple_fields(){
        unsafe{crate::init()};
        let mut ctx = FakeContext::new();
        assert_eq!(ctx.add_text_field("title".to_string(), 2, true), 0);
        assert_eq!(ctx.add_text_field("body".to_string(), 2, true), 1);
        assert_eq!(ctx.add_date_field("date".to_string(), 2, true), 2);
        assert_eq!(ctx.add_u64_field("someu64".to_string(), 2, true), 3);
        assert_eq!(ctx.add_i64_field("somei64".to_string(), 2, true), 4);
        assert_eq!(ctx.add_f64_field("somef64".to_string(), 2, true), 5);
    }
}
