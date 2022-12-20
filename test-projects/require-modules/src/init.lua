local ProcessService = game:GetService("ProcessService")

local foo = script.foo

for i = 1, 10 do
	print(i)

	if i == 6 then
		print("Reached limit, exiting process")
		ProcessService:ExitAsync(0)
	end

	task.wait(0.1)
end
