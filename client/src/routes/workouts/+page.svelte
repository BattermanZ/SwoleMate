<script lang="ts">
	const tableData = {
		head: ['Date', 'Exercise', 'Sets', 'Reps', 'Weight', 'Notes'],
		body: [
			['2024-02-04', 'Bench Press', '3', '10', '135 lbs', 'Feeling strong'],
			['2024-02-04', 'Squats', '4', '8', '225 lbs', 'New PR!'],
			['2024-02-04', 'Deadlift', '3', '5', '315 lbs', 'Working on form']
		]
	};
</script>

<div class="space-y-8">
	<header class="flex justify-between items-center">
		<h2 class="h2">Workouts</h2>
		<a href="/workouts/new" class="btn variant-filled-primary">
			<span>New Workout</span>
		</a>
	</header>

	<div class="card p-4">
		<div class="flex justify-between items-center pb-4">
			<div class="space-x-2">
				<button class="btn variant-soft">Filter</button>
				<button class="btn variant-soft">Sort</button>
			</div>
			<div class="relative">
				<input
					class="input"
					type="search"
					placeholder="Search workouts..."
				/>
			</div>
		</div>

		<div class="table-container">
			<table class="table table-hover">
				<thead>
					<tr>
						{#each tableData.head as header}
							<th>{header}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each tableData.body as row}
						<tr>
							{#each row as cell}
								<td>{cell}</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</div> 