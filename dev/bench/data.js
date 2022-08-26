window.BENCHMARK_DATA = {
  "lastUpdate": 1661546120258,
  "repoUrl": "https://github.com/toposware/winterfell",
  "entries": {
    "Rust Benchmark": [
      {
        "commit": {
          "author": {
            "name": "toposware",
            "username": "toposware"
          },
          "committer": {
            "name": "toposware",
            "username": "toposware"
          },
          "id": "d83893d342cd3f9dbf8dd505d2f2ad26381eb1f9",
          "message": "ci: add benchmarks",
          "timestamp": "2022-08-20T21:50:10Z",
          "url": "https://github.com/toposware/winterfell/pull/36/commits/d83893d342cd3f9dbf8dd505d2f2ad26381eb1f9"
        },
        "date": 1661546119327,
        "tool": "cargo",
        "benches": [
          {
            "name": "syn_div/high_degree/262144",
            "value": 2489879,
            "range": "± 45541",
            "unit": "ns/iter"
          },
          {
            "name": "syn_div/high_degree/524288",
            "value": 5007195,
            "range": "± 100903",
            "unit": "ns/iter"
          },
          {
            "name": "syn_div/high_degree/1048576",
            "value": 13011634,
            "range": "± 125409",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}