#!/usr/bin/env python3
# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "toml",
# ]
# ///
"""
Introspect rank-eval to discover what's actually implemented vs documented.

This script analyzes the rank-eval codebase to:
1. Extract all implemented metrics
2. Extract all dataset loaders
3. Extract all features/modules
4. Compare with README documentation
5. Generate comprehensive report

The "helm" design: rank-rank is central control, can introspect all rank-* repos.
"""

import ast
import re
import sys
from pathlib import Path
from typing import Dict, List, Set, Tuple
import json

def find_rust_files(repo_path: Path) -> List[Path]:
    """Find all Rust source files."""
    return list(repo_path.rglob("src/**/*.rs")) + list(repo_path.rglob("tests/**/*.rs"))

def extract_public_functions(content: str) -> List[str]:
    """Extract public function names from Rust code."""
    functions = []
    # Match: pub fn function_name
    pattern = r'pub\s+fn\s+(\w+)'
    for match in re.finditer(pattern, content):
        functions.append(match.group(1))
    return functions

def extract_public_structs(content: str) -> List[str]:
    """Extract public struct names from Rust code."""
    structs = []
    # Match: pub struct StructName
    pattern = r'pub\s+struct\s+(\w+)'
    for match in re.finditer(pattern, content):
        structs.append(match.group(1))
    return structs

def extract_public_enums(content: str) -> List[str]:
    """Extract public enum names from Rust code."""
    enums = []
    # Match: pub enum EnumName
    pattern = r'pub\s+enum\s+(\w+)'
    for match in re.finditer(pattern, content):
        enums.append(match.group(1))
    return enums

def extract_modules(content: str) -> List[str]:
    """Extract module declarations."""
    modules = []
    # Match: pub mod module_name
    pattern = r'pub\s+mod\s+(\w+)'
    for match in re.finditer(pattern, content):
        modules.append(match.group(1))
    return modules

def analyze_rank_eval(repo_path: Path) -> Dict:
    """Analyze rank-eval codebase comprehensively."""
    results = {
        "metrics": {
            "binary": [],
            "graded": [],
            "statistics": [],
        },
        "datasets": {
            "loaders": [],
            "types": [],
        },
        "modules": [],
        "structs": [],
        "enums": [],
        "features": [],
        "exports": [],
    }
    
    rust_files = find_rust_files(repo_path)
    
    for file_path in rust_files:
        try:
            content = file_path.read_text()
            relative_path = file_path.relative_to(repo_path)
            
            # Extract based on file location
            if "binary.rs" in str(relative_path):
                functions = extract_public_functions(content)
                results["metrics"]["binary"].extend(functions)
            elif "graded.rs" in str(relative_path):
                functions = extract_public_functions(content)
                results["metrics"]["graded"].extend(functions)
            elif "statistics.rs" in str(relative_path):
                functions = extract_public_functions(content)
                results["metrics"]["statistics"].extend(functions)
            elif "loaders.rs" in str(relative_path):
                functions = extract_public_functions(content)
                results["datasets"]["loaders"].extend(functions)
                enums = extract_public_enums(content)
                results["datasets"]["types"].extend(enums)
            
            # Extract modules from lib.rs
            if "lib.rs" in str(relative_path):
                modules = extract_modules(content)
                results["modules"].extend(modules)
                
                # Extract re-exports
                re_export_pattern = r'pub\s+use\s+([\w:]+)'
                for match in re.finditer(re_export_pattern, content):
                    results["exports"].append(match.group(1))
            
            # Extract all structs and enums
            structs = extract_public_structs(content)
            results["structs"].extend(structs)
            enums = extract_public_enums(content)
            results["enums"].extend(enums)
            
        except Exception as e:
            print(f"Warning: Could not analyze {file_path}: {e}", file=sys.stderr)
    
    # Deduplicate
    for key in results:
        if isinstance(results[key], list):
            results[key] = sorted(list(set(results[key])))
        elif isinstance(results[key], dict):
            for subkey in results[key]:
                if isinstance(results[key][subkey], list):
                    results[key][subkey] = sorted(list(set(results[key][subkey])))
    
    return results

def read_readme(repo_path: Path) -> str:
    """Read README content."""
    readme_path = repo_path / "README.md"
    if readme_path.exists():
        return readme_path.read_text()
    return ""

def extract_readme_metrics(readme: str) -> Set[str]:
    """Extract metrics mentioned in README."""
    metrics = set()
    
    # Look for metric lists
    # Match `metric` or `metric()`
    # Look for metric lists
    # Match `metric` or `metric()`
    # Added statistical functions to the allowed list so they populate in the report
    pattern = r'`(\w+_at_k|mrr|ndcg|map|err|rbp|f_measure|success|r_precision|precision|recall|dcg|idcg|average_precision|cohens_d|paired_t_test|confidence_interval|compute_ndcg|compute_map)(?:\(\))?`'
    for match in re.finditer(pattern, readme, re.IGNORECASE):
        match_str = match.group(1).lower()
        metrics.add(match_str)
    
    # Also look for "Available binary metrics:" section
    if "Available binary metrics:" in readme:
        section = readme.split("Available binary metrics:")[1].split("###")[0]
        for line in section.split("\n"):
            if "`" in line:
                metric_match = re.search(r'`(\w+)`', line)
                if metric_match:
                    metrics.add(metric_match.group(1).lower())
    
    return metrics

def extract_readme_datasets(readme: str) -> Set[str]:
    """Extract datasets mentioned in README."""
    datasets = set()
    
    # Look for dataset mentions
    pattern = r'(MS\s+MARCO|BEIR|MIRACL|MTEB|HotpotQA|Natural\s+Questions|SQuAD|TREC)'
    for match in re.finditer(pattern, readme, re.IGNORECASE):
        datasets.add(match.group(1).lower().replace(" ", "_"))
    
    return datasets

def generate_report(analysis: Dict, readme: str, repo_path: Path) -> str:
    """Generate comprehensive introspection report."""
    readme_metrics = extract_readme_metrics(readme)
    readme_datasets = extract_readme_datasets(readme)
    
    # Collect all implemented metrics
    implemented_metrics = set()
    implemented_metrics.update(analysis["metrics"]["binary"])
    implemented_metrics.update(analysis["metrics"]["graded"])
    implemented_metrics.update(analysis["metrics"]["statistics"])
    
    # Normalize for comparison
    normalized_impl = {m.lower().replace("_", "_") for m in implemented_metrics}
    normalized_readme = {m.lower().replace("_", "_") for m in readme_metrics}
    
    # Find gaps
    missing_in_readme = normalized_impl - normalized_readme
    extra_in_readme = normalized_readme - normalized_impl
    
    report = []
    report.append("# rank-eval Comprehensive Introspection Report\n")
    report.append("Generated by rank-rank introspection system (helm design)\n")
    report.append("=" * 80 + "\n")
    
    # Metrics section
    report.append("## Metrics\n")
    report.append(f"**Total Implemented**: {len(implemented_metrics)}\n")
    report.append(f"**In README**: {len(readme_metrics)}\n")
    report.append(f"**Missing from README**: {len(missing_in_readme)}\n")
    report.append(f"**Extra in README**: {len(extra_in_readme)}\n\n")
    
    report.append("### Binary Metrics\n")
    for metric in sorted(analysis["metrics"]["binary"]):
        in_readme = "✅" if metric.lower() in normalized_readme else "❌"
        report.append(f"- {in_readme} `{metric}()`\n")
    
    report.append("\n### Graded Metrics\n")
    for metric in sorted(analysis["metrics"]["graded"]):
        in_readme = "✅" if metric.lower() in normalized_readme else "❌"
        report.append(f"- {in_readme} `{metric}()`\n")
    
    report.append("\n### Statistical Functions\n")
    for func in sorted(analysis["metrics"]["statistics"]):
        in_readme = "✅" if func.lower() in normalized_readme else "❌"
        report.append(f"- {in_readme} `{func}()`\n")
    
    # Datasets section
    report.append("\n## Dataset Loaders\n")
    report.append(f"**Total Implemented**: {len(analysis['datasets']['loaders'])}\n")
    report.append(f"**Dataset Types**: {len(analysis['datasets']['types'])}\n\n")
    
    for loader in sorted(analysis["datasets"]["loaders"]):
        report.append(f"- `{loader}()`\n")
    
    report.append("\n### Dataset Types\n")
    for dtype in sorted(analysis["datasets"]["types"]):
        report.append(f"- `{dtype}`\n")
    
    # Modules section
    report.append("\n## Modules\n")
    for module in sorted(analysis["modules"]):
        report.append(f"- `{module}`\n")
    
    # Gaps section
    if missing_in_readme:
        report.append("\n## ⚠️ Metrics Missing from README\n")
        for metric in sorted(missing_in_readme):
            report.append(f"- `{metric}`\n")
    
    if extra_in_readme:
        report.append("\n## ⚠️ Metrics in README but Not Found in Code\n")
        for metric in sorted(extra_in_readme):
            report.append(f"- `{metric}`\n")
    
    # Summary
    report.append("\n## Summary\n")
    report.append(f"- **Modules**: {len(analysis['modules'])}\n")
    report.append(f"- **Public Structs**: {len(analysis['structs'])}\n")
    report.append(f"- **Public Enums**: {len(analysis['enums'])}\n")
    report.append(f"- **Re-exports**: {len(analysis['exports'])}\n")
    
    coverage = (len(readme_metrics) / len(implemented_metrics) * 100) if implemented_metrics else 0
    report.append(f"- **Documentation Coverage**: {coverage:.1f}%\n")
    
    return "".join(report)

def main():
    if len(sys.argv) < 2:
        print("Usage: introspect_rank_eval.py <rank-eval-path>")
        sys.exit(1)
    
    repo_path = Path(sys.argv[1])
    if not repo_path.exists():
        print(f"Error: Path does not exist: {repo_path}")
        sys.exit(1)
    
    print("Analyzing rank-eval codebase...", file=sys.stderr)
    analysis = analyze_rank_eval(repo_path)
    
    print("Reading README...", file=sys.stderr)
    readme = read_readme(repo_path)
    
    print("Generating report...", file=sys.stderr)
    report = generate_report(analysis, readme, repo_path)
    
    print(report)
    
    # Also output JSON for programmatic use
    output_path = repo_path / "INTROSPECTION_REPORT.json"
    with open(output_path, "w") as f:
        json.dump({
            "analysis": analysis,
            "summary": {
                "total_metrics": len(analysis["metrics"]["binary"]) + len(analysis["metrics"]["graded"]) + len(analysis["metrics"]["statistics"]),
                "total_datasets": len(analysis["datasets"]["loaders"]),
                "total_modules": len(analysis["modules"]),
            }
        }, f, indent=2)
    
    print(f"\nJSON report saved to: {output_path}", file=sys.stderr)

if __name__ == "__main__":
    main()

