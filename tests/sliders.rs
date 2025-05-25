// use git_repository::bstr::BStr;
// use git_repository::objs::Kind;
// use git_repository::Repository;
// use imara_diff::{intern::InternedInput, Algorithm, Diff};

// fn diff(algo: Algorithm, repo: &Repository, file_rev1: &BStr, file_rev2: &BStr) {
//     let file1 = repo
//         .rev_parse_single(file_rev1)
//         .unwrap()
//         .object()
//         .unwrap()
//         .peel_to_kind(Kind::Blob)
//         .unwrap();
//     let file2 = repo
//         .rev_parse_single(file_rev2)
//         .unwrap()
//         .object()
//         .unwrap()
//         .peel_to_kind(Kind::Blob)
//         .unwrap().kind;

//     let input = InternedInput::new(&*file1.data, &*file2.data);
//     let mut diff = Diff::compute(algo, &input);
//     diff.postprocess(&input);
//     diff.unified_diff(,)
// }
