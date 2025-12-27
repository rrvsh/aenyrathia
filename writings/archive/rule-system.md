# Rule System

Pinbreak's main tenet is player agency.
The rules and mechanics described below should be ruthlessly optimised in service of this main tenet, and should not include anything that detracts from it.
The key concepts are that of designing the game around the players (including the DM) and the characters.

## Characters

The player characters are the lifeblood of Pinbreak. The focus should be on exploring their stories. This includes the DM, though their "player character" is the world.
Each player has **full agency** over what their characters do and what happens to them, unless they explicitly allow another player to control them.

### Domains

Any given character in a story should have their areas of expertise, proficiency, talent, or otherwise. Pinbreak defines these as the Domains each character has.

Domains are purposely vague - one could say their character is an expert in something more nebulous like the domain of physical endurance, or more specific like fire magic.
However, all players at the table must agree on the domain making sense for that character. The point of this vagueness is to allow the player more flexibility in defining their character.

Each domain should have a die size allocated to it, representing the character's level in that domain. This abstraction is to make it easier for a player to determine
what die size to use when making an action that is in line with that domain, and thus should represent a character's proficiency or talent in that domain.

A brief guide to choosing what die size should map to which domain is:

- d4: Untrained -> Average person trying to get through a locked door
- d6: Adept -> A fit human climbing the side of a rock wall
- d8: Expert -> A veteran officer planning a military tactic
- d10: Master -> An archmage who is considered the best in their field 
- d12: Legendary -> A demigod, or someone blessed by a powerful magical being.
- d20 -> The powerful magical being.

A good mental model for this is that the percentage of the population at each level for each domain should go down by a few orders of magnitude as the dice size goes up.
For example, 100% of the population would be untrained in rock-climbing, 1% could be adept, 0.01% could be expert, 0.0001% could be master, 0.000001% could be legendary,
and only gods or other powerful magical beings could use a d20. In practise, a player character should be between a d6 to d10, with rare d12s.

#### Strain

A character can accrue Strain on any of their domains. This represents pushing beyond their limits on that domain, though this is only possible in service of a goal beyond just improvement.
A character accrues Strain by initiating a Push during an Exchange. Afterwards, they can no longer push until the Strain is rectified.

A character can rectify Strain by taking time to rest and recuperate at any point after the Scene where they chose to Push, but not during the same Scene.
This can take the form of simple time off, or could be the character solidifying their understanding of the new ability or proficiency in that domain that they gained.

When strain is rectified on a domain, the dice size stays the same, but the number of dice increases by 1 up to a max of 3.

In practise, a character's character sheet could contain the following section:

| Domains            | Die  | Strained |
| ------------------ | ---- | -------- |
| Physical Endurance | 2d6  | ✓        |
| Fire Magic         | 2d8  |          |
| ------------------ | ---  | -------- |

### Actions

The main mechanism a character has to effect change on the world are their actions, and these actions are the primary thing a player character should be focused on.

An action should always serve an objective, no matter how mundane. Every action has a likelihood of success, and for actions that have a lower likelihood of success, the player
must roll dice to determine the outcome. Actions can also be opposed by other characters, either passively or actively. When success or failure is certain, the player should not roll.

When taking any action that is opposed or where success is uncertain, the player must declare what dice they will roll based on their likelihood of success, taking into account their
domains, the context of the scene, and any other contributing factors.

Actions a character can take are not restricted to the domains they have on their sheet, and the dice roll for an action is similarly not required to align with one of their domains.
Instead, the domains a character has access to should guide the choices they make during a scene, and provide structure for what dice rolls fit what actions.

## Sessions

Sessions will be divided into Scenes, explored below. Each session should follow a strict structure, to take mental load off of the players and let them focus on having fun.

Sessions should start with a description of the context of the session, usually in the form of a brief recap of the story thus far.
This should be followed by a more detailed recap of the previous session, if any.

```
We're starting in AA 0.6, 6 months after the Seraphs killed the Great Ape and fired the Extraplanar Resonator. Since this happened, extraplanar incursions have been happening
at a stellar rate across Aenyrathia, leaking demons of all shapes and sizes into the world. The most pertinent effects of these incursions to our session today are twofold.
Firstly, EchoTech, as enterprising as ever, has formed a ninth division to do research into these incursions.
Secondly, a few people with superpowers have begun emerging in the wake of the more powerful incursions, though most of them have disappeared or been disappeared.
You will be playing as a group of D:IX researchers who, after prolonged exposure to incursions during your research, have not only developed some of these powers, but done so
in a way that has mutated you to the point of monstrousness.
The story will pick up as you head to the site of a potential incursion in Sector 5. Be on guard, though - insurgents from the newly separated Sector 7 have been spotted in the area.
```

Each non-DM player will then describe the character(s) they are playing that session.
This description should be a general overview of the character as the player understands them, and their current state of mind.
For example:

```
I'll be playing Zel'eon, a young Mosskachi who is obsessed with improving themselves through combat.
I'm buzzing from the excitement of just having incited rebellion against EchoTech in Rath Zidul and I wish I could stay there and help Dregdul and Laura, but I know I have a responsibility
to finish what I started in Ilrinia.
```

With that, the first Scene should begin.

## Scenes

As mentioned above, the Scene is the first narrative building block used in Pinbreak. Each scene should involve one or more parties, or groups of characters with a shared objective.

Each scene will begin with an intermezzo. This should be a narratively quieter moment to allow the characters to breathe and for all players to be in alignment.
The intermezzo should include the GM setting the scene with a description of where the characters are, painting a mental picture for everyone.
Each player should then describe their characters as the other characters would see them, focusing on drawing as clear a mental picture of themselves as possible, including clothing,
demeanour, and any other physical attributes noticeable. They should then describe what is going on in the mind of their character, with regards to the upcoming scene.

```
DM: It's a quiet night in Ilrinia. You are each strapped into your seats in a quadcopter, flying high above the city on your way to meet up with Senzorin in the forest of Alwahd.
    The lights in the quadcopter's cabin are dimmed, casting a faint red glow over the scene. How is everyone feeling?
Zel'eon: You see me, a half-elf in his late teens wearing my mom's lab coat over my white orichalcum armour plating. I'm looking out the window of the quadcopter, my mind still in Mosskach.
         I wish I could be there supporting Dregdul and Laura.
Arche: I'm standing in the cockpit, talking to the pilot. I'm a young sun elf woman, dressed in ostentatious white plate armour that we just got from the Council. I'm clearly still
       uncomfortable in it, shifting to and fro and knocking into things without realising it. I'm excited to be back in my home city, and especially to see Senzo again.
```

The scene will then start. There are three ways a scene can progress:

1. The DM prompts the players for what they want to do. The players can then ask questions, or state actions they want to take. For example:

```
DM: Well, you have some time before you get there. Is there any conversation going on?
Arche: Noticing Zel's mind is elsewhere, I'd like to go up to him.
Zel'eon: I look up as Arche approaches and offer a weak smile.
Arche: Focused on Rath Zidul, huh?
```

2. The DM can state something that happens in a scene, and then ask the players how they react. For example:

```
DM: Zel, as you're looking out the window, you notice that a flock of birds seems to be flying directly towards the quadcopter. As they get closer, though, you realise that you were mistaken -
    these aren't birds. A group of wood elves, clearly some sort of eagle tribe, are flying towards you, and each of them seems to be armed. What do you do?
Zel'eon: I burst out of my seat. How long do we have?
DM: From their speed, looks like not much - they should intercept in 30 seconds.
Zel'eon: Arche, we got incoming!
Arche: I'm gonna grab the steering controls and make a hard left.
```

3. The players can declare an action they want to take, optionally asking the DM for more information first. For example:

```
Arche: I'd like to ask the pilot if I could have a turn at the controls. Do they look amenable to this, or should I butter them up first?
DM: The pilot seems pretty nervous to be flying for the Seraphs. You could probably charm him into it.
```

Scenes will continue as above until a satisfying resolution is achieved. A scene resolution usually takes the form of a party achieving their objective, or having the situation change enough
that the objective must too.

```
DM: As you wrap up your conversation, the pilot calls out to you, saying you should be strapped in for the landing sequence. Do you take your seats?
Zel'eon: Yeah, me and Arche sit down and strap in.
DM: You slowly begin spiralling down amongst the tall trees of Alwahd.
Senzorin: Anyone wanna look out the window?
Arche: Yes? What do I see?
Senzorin: You see me, motherfucker! I'm riding on Miranai to escort you down.
```

The scene will then end. Before proceeding with the next, there should be an additional intermezzo where the DM should describe the scene as it now stands.
Each player will then, in turn, describe how they look after and feel about the events of the scene.

```
DM: Zel'eon, as you withdraw your blade from the dead wood elf's chest, his corpse falls backward off the open cabin door of the copter. As you look around, you see blood splattered across the cabin,
    most of it from the quadcopter pilot, who lays slumped against one of the seats. Arche is tending to him using her healing magic. How do you feel?
Zel'eon: I feel ready to tear the Panther limb from limb. I walk over to the cockpit and descend into my mind, telling my body to take over and fly us to Alwahd. As my eyes glaze over, the armor plates
         on my arms separate, tendrils of blood emerging from them to hook into the quadcopter's control console.
Arche: As I finish stabilising the pilot, I sit back, covered in his blood up to my elbows. I sigh, and look out the window, wishing all of this could just be over.
```

Take a break before the next scene, and take some time for each player to check in on how they feel so far. During this break, the players should quickly note down anything important from the scene that
they learned or gained, or might be important for upcoming scenes. Taking notes during scenes is discouraged, but documentation in between is highly recommended.

## Exchanges

A scene may sometimes call for a more structured system of actions. An example for this would be a scene that includes combat between two or more parties.
In these cases, the scene will include an Exchange. An exchange must always involve at least two parties, each with a clear objective.

### Anatomy of an Exchange

Each round of an exchange will involve one Active and at least one Opposing side. The first party to take the active side is determined by the circumstances of the scene.
For example, if a party surprises another, they will take the active side first.

Each character on the Active side will declare what action they want to take towards their objective, and the dice they will roll to determine their success.
Then, each character on the opposing side will declare what they are doing to oppose the other side's objective, and the dice they will roll.

The acting side will then roll their respective dice. They will declare the highest number rolled amongst all players.
The opposing side will respond by rolling their own, and declaring their highest number rolled.
If the acting side's highest number rolled is equal to or higher than the opposing side's highest number rolled, the actions are considered a success.
Otherwise, the opposing side successfully thwarts the acting side, and they make no progress towards their objective.
Then, the exchange continues with the next party being the acting side.

An exchange is considered resolved the same way a scene is - upon all parties either meeting their objective or the situation changing enough that the objective must too.

### Pushing

At any point during a round, a player may declare their intention to Push, representing the character pushing beyond their limits to achieve their objective.
If the rest of the party agrees with this, that player will be elected as the Champion, and the other side must then elect their own Champion.

The Champions on either side will declare the action they are taking and the dice they will roll, with the restriction that it **must** align with one of their domains.
This action should be something novel or difficult for the character, representing them doing anything they can to achieve their goal.
The other players in their party will declare how they are helping the Champion achieve that action, and declare their own dice rolls.

The Champions will then take all the dice from the other players, and roll all of them together against the other side.
The result is then compared as per normal, with the acting side succeeding on their highest number rolled being equal to or higher than the other sides'.

However, if there are **any** 1s rolled on any of the dice, the action will result in a complication. The Champion who rolled the 1 must then describe why and how
the complication occurs. 

Additionally, regardless of the number of 1s rolled, the Champion will accrue Strain on the domain they aligned their action with **if the action succeeds**.
If the action failed, the character is considered to have found their current limit for that domain, though they are welcome to continue pushing.
