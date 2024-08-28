import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Link, Outlet, createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { z } from 'zod';
import { useAuth } from "@clerk/clerk-react";
import { NavigationMenu, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, navigationMenuTriggerStyle } from "@/components/ui/navigation-menu";
import { cn } from "@/lib/utils";

export const Route = createFileRoute('/_app/_workbench/teams')({
    component: Teams,

    loader: async () => {
        const url = await invoke("get_server_url");
        return {
            url: url
        }
    }
})

interface Team {
    id: number,
    name: string
}

function Teams() {
    const { url } = Route.useLoaderData();
    const [formMessage, setFormMessage] = useState("");
    const [proposedTeamName, setProposedTeamName] = useState("");
    const [created, setCreated] = useState(false);
    const { getToken } = useAuth();
    const { isPending, isError, data, error } = useQuery({
        queryKey: ['teams'],
        queryFn: async () => {
            const endpoint = (url as string) + "/team"
            const response = await fetch(endpoint, {
                headers: {
                    Authorization: `Bearer ${await getToken()}`
                },
                method: "GET",
                mode: "cors"
            });
            return response.json();
        }
    })
    const queryClient = useQueryClient();

    const mutation = useMutation({
        mutationFn: async () => {
            const endpoint = (url as string) + "/team"
            return await fetch(endpoint, {
                method: "POST",
                mode: "cors",
                body: JSON.stringify({
                        "name": proposedTeamName
                }),
                headers: { Authorization: `Bearer ${await getToken()}` }
            })
        },
        onError: (error) => {
            setFormMessage(error.message)
        },
        onSuccess: async (res) => {
            const data = await res.json()
            if(data.status === "success") {
                setFormMessage("Team created successfully.");
                setCreated(true);
            }

            queryClient.invalidateQueries({
                queryKey: ['teams', 'projects']
            })
        }
    })

    function createTeam() {
        setFormMessage("");

        // allow for alphanumeric and whitespace
        const input = z.string().regex(new RegExp('^[a-zA-Z0-9 ]+$'));
        if(input.safeParse(proposedTeamName).success) {
            const data = mutation.mutate();
        }
        else {
            setFormMessage("Team name invalid.")
        }
    }

    if(isPending) {
        // TODO proper skeleton
        return (
            <div>
                Loading...
            </div>
        )
    }
    else if(isError) {
        return (
            <div>an error occured while fetching data :c</div>
        )
    }
    else if(data.response != "success") {
        return (
            <div>an error occured while fetching data :c</div>
        )
    }
    console.log(data)

    return (
        <div className="flex flex-row space-x-4">
            <div className="w-fit">
                <h1 className="text-2xl font-semibold text-center">Teams</h1>
                    <NavigationMenu className="max-w-1/4 p-2">
                    <ScrollArea className="h-48">
                        <NavigationMenuList className="grid grid-flow-row items-center space-y-2 py-2">
                            {
                                data.body.teams && !isPending && !isError ?
                                data.body.teams.map((team: Team) => 
                                    <NavigationMenuItem className="w-48 text-center text-wrap" key={team.id}>
                                        <NavigationMenuLink className={cn(navigationMenuTriggerStyle(), "min-w-full")} asChild>
                                            <Link to="/teams/$teamid" params={{ teamid: String(team.id) }}>{team.name}</Link>
                                        </NavigationMenuLink>
                                    </NavigationMenuItem>
                                ) : <div>No teams found</div>
                            }
                        </NavigationMenuList>
                    </ScrollArea>
                    </NavigationMenu>
                    <Dialog onOpenChange={() => {setFormMessage(""); setCreated(false)}}>
                    <DialogTrigger asChild>
                        <Button variant={"outline"} className="w-full" disabled={!data.open}>Create Team</Button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Create a new Team</DialogTitle>
                        </DialogHeader>
                        <Input placeholder="Team Name" onChange={(e) => setProposedTeamName(e.target.value)}/>
                        <div className="">{formMessage}</div>
                        <DialogFooter>
                            <Button onClick={createTeam} disabled={mutation.isPending || created}>Create</Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>
            <Outlet />
        </div>
    )
}