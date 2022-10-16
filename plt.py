import json
from pathlib import Path
import numpy as np
from DMT.core import Plot
import subprocess

base_dir = Path(__file__).parent.resolve()
target_dir = base_dir / "target" / "criterion"
for similar in [True]:
    for repo in target_dir.glob("*"):

        repo_name = repo.name
        if not repo_name.endswith("_plot"):
            continue
        repo_name = repo_name[:-5]

        plot = Plot(
            f"{repo_name}_comparison",
            x_label=r"$(M + N) D$",
            y_label=r"$T \left(\si{\milli\second}\right)$",
            y_scale=1e-3,
            y_log=True,
            x_log=True,
            legend_location="upper left",
        )

        for algorithm in repo.glob("*"):
            if "similar" in algorithm.name and not similar:
                continue

            data = []
            for dir in algorithm.glob("*"):
                if "__" not in dir.name:
                    continue
                path = dir / "new" / "estimates.json"
                dir_components = dir.name.split("__")
                complexity = int(dir_components[0])
                scale = int(dir_components[1])
                mean = json.loads(path.read_text())["mean"]
                estimate = mean["point_estimate"] / scale
                lower_bound = mean["confidence_interval"]["lower_bound"] / scale
                upper_bound = mean["confidence_interval"]["upper_bound"] / scale
                data.append((complexity, estimate, lower_bound, upper_bound))

            algorithm = algorithm.name
            data.sort(key=lambda it: it[0])
            complexity = np.array([c for c, _, _, _ in data])
            estimate = np.array([it for _, it, _, _ in data])
            lower_bound = np.array([it for _, _, it, _ in data])
            upper_bound = np.array([it for _, _, _, it in data])

            plot.add_data_set(
                complexity,
                estimate,
                label=algorithm.replace("_", "-"),
            )
        plot.save_tikz("plots", standalone=True, build=True, width="0.83\\textwidth")

for repo in target_dir.glob("*"):
    repo_name = repo.name
    if not repo_name.endswith("_plot"):
        continue
    repo_name = repo_name[:-5]
    plot = Plot(
        f"{repo_name}_speedup",
        x_label=r"$(M + N) D$",
        y_label=r"$\frac{T_\mathrm{myers}}{T_\mathrm{histogram}}$",
        # y_scale=1e-3,
        # y_log=True,
        x_log=True,
        legend_location="upper left",
    )

    algorithm_data = {}

    for algorithm in repo.glob("*"):
        if "similar" in algorithm.name:
            continue

        data = []
        for dir in algorithm.glob("*"):
            if "__" not in dir.name:
                continue
            path = dir / "new" / "estimates.json"
            dir_components = dir.name.split("__")
            complexity = int(dir_components[0])
            scale = int(dir_components[1])
            mean = json.loads(path.read_text())["mean"]
            estimate = mean["point_estimate"] / scale
            lower_bound = mean["confidence_interval"]["lower_bound"] / scale
            upper_bound = mean["confidence_interval"]["upper_bound"] / scale
            data.append((complexity, estimate, lower_bound, upper_bound))

        algorithm = algorithm.name
        data.sort(key=lambda it: it[0])
        complexity = np.array([c for c, _, _, _ in data])
        estimate = np.array([it for _, it, _, _ in data])
        lower_bound = np.array([it for _, _, it, _ in data])
        upper_bound = np.array([it for _, _, _, it in data])
        algorithm_data[algorithm] = complexity, estimate
    myers = algorithm_data["imara_diff-myers"]
    histogram = algorithm_data["imara_diff-histogram"]
    speedup = myers[1] / histogram[1]
    plot.add_data_set(complexity, speedup)
    plot.save_tikz("plots", standalone=True, build=True, width="0.83\\textwidth")


for file in (base_dir / "plots").glob("*.pdf"):
    subprocess.run(["pdf2svg", file, file.with_suffix(".svg")])
# plt.show()
