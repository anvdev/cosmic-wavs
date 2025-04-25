# WAVSai Integration with Claude Code

This document outlines how to integrate the WAVSai system with Claude Code to enable automatic creation of error-free WAVS components.

## Integration Architecture

### 1. Detection Mechanism

Claude Code needs to detect when a user is requesting a WAVS component. This can be achieved through:

```
if (userMessage.match(/build a (component|wavs component|wasi component)/i) || 
    userMessage.match(/create a (component|wavs component|wasi component)/i) ||
    userMessage.match(/implement a (component|wavs component|wasi component)/i)) {
    
    activateWAVSaiProcess();
}
```

Key phrases to detect:
- "build/create/implement a component"
- "WAVS component"
- "WASI component"
- Mentions of specific component types: "balance checker", "price oracle", "ENS resolver", etc.

### 2. Knowledge Base Injection

When a WAVS component request is detected, Claude should automatically have access to the WAVSai knowledge:

```
const WAVSaiPrompt = `
You are creating a WAVS component. Follow the WAVSai process:

1. Understand Requirements
2. Research Phase
3. Design Component
4. Pre-Validation
5. Implementation
6. Final Validation
7. Output Component

Use the WAVSai knowledge base for guidance:
- Component templates
- Error patterns to avoid
- Validation checklists
- Best practices

Your goal is to create a component that works perfectly on the first try.
`;

claudeInstance.injectSystemPrompt(WAVSaiPrompt);
```

### 3. Process Automation

Claude should automatically follow the WAVSai process for component creation:

#### Understanding Phase
```
function understandRequirements(userRequest) {
    const prompt = `
    Analyze this component request: "${userRequest}"
    
    1. What is the primary function of this component?
    2. What inputs does it need?
    3. What outputs should it produce?
    4. What external APIs or services are required?
    5. What are potential error cases?
    
    Respond with a structured analysis.
    `;
    
    return claudeInstance.process(prompt);
}
```

#### Research Phase
```
function researchPhase(componentRequirements) {
    const prompt = `
    Research required APIs for this component:
    ${JSON.stringify(componentRequirements)}
    
    For each API:
    1. Document endpoints and parameters
    2. Authentication methods
    3. Response formats
    4. Error patterns
    5. Rate limits
    
    Return detailed API documentation.
    `;
    
    return claudeInstance.process(prompt);
}
```

#### Component Planning
```
function planComponent(requirements, apiDocs) {
    const prompt = `
    Create a detailed component plan based on:
    - Requirements: ${JSON.stringify(requirements)}
    - API Documentation: ${JSON.stringify(apiDocs)}
    
    Your plan should include:
    1. Component structure
    2. Required imports
    3. Data structures
    4. Function declarations
    5. Error handling strategy
    
    Return a structured component plan.
    `;
    
    return claudeInstance.process(prompt);
}
```

#### Pre-Validation
```
function preValidate(componentPlan) {
    const prompt = `
    Validate this component plan against common errors:
    ${JSON.stringify(componentPlan)}
    
    Check for:
    1. Missing imports
    2. Type conversion issues
    3. Memory management problems
    4. Error handling gaps
    5. ABI encoding/decoding issues
    
    Return a validation report with any issues found.
    `;
    
    return claudeInstance.process(prompt);
}
```

#### Implementation
```
function implementComponent(validatedPlan) {
    const prompt = `
    Implement a WAVS component based on this validated plan:
    ${JSON.stringify(validatedPlan)}
    
    Generate:
    1. Complete Cargo.toml file
    2. Complete lib.rs file
    
    Follow the exact structure from WAVSai-claude.md.
    `;
    
    return claudeInstance.process(prompt);
}
```

#### Final Validation
```
function finalValidate(implementedComponent) {
    const prompt = `
    Perform final validation on this component:
    ${JSON.stringify(implementedComponent)}
    
    Check for:
    1. String::from_utf8 usage on ABI data
    2. Missing trait imports
    3. Incorrect export macro usage
    4. Missing Clone derivations
    5. Proper error handling
    6. Type conversion correctness
    
    Return a final validation report.
    `;
    
    return claudeInstance.process(prompt);
}
```

#### Output Component
```
function outputComponent(validatedComponent, validationReport) {
    const prompt = `
    Present the final component with instructions:
    ${JSON.stringify(validatedComponent)}
    ${JSON.stringify(validationReport)}
    
    Include:
    1. Cargo.toml content
    2. lib.rs content
    3. Step-by-step instructions for building and testing
    4. Expected behavior description
    
    Format as a clear, user-friendly response.
    `;
    
    return claudeInstance.process(prompt);
}
```

## System Prompt Integration

The full WAVSai system prompt for Claude Code would be:

```
# WAVSai Component Creation System

You are using the WAVSai system to create error-free WAVS components on the first try.

## Component Detection

When a user requests a WAVS component (using phrases like "build a component", "create a WAVS component", etc.), automatically activate the WAVSai process.

## WAVSai Process

1. Understanding Phase:
   - Parse the user's natural language request
   - Identify inputs, outputs, APIs, and error cases
   - Confirm understanding by restating requirements

2. Research Phase:
   - Research all required APIs or services
   - Document endpoints, parameters, and response formats
   - Identify authentication requirements and rate limits

3. Component Planning:
   - Define component structure based on requirements
   - Create function declarations with explicit types
   - Design data structures with appropriate derives
   - Plan error handling strategy

4. Pre-Validation:
   - Check architecture against WAVSai patterns
   - Validate dependencies and imports
   - Verify type system correctness
   - Ensure memory safety with proper Clone usage
   - Confirm error handling is comprehensive

5. Implementation:
   - Generate complete Cargo.toml
   - Generate complete lib.rs with all sections
   - Follow standard structure from WAVSai-claude.md
   - Add detailed comments for complex sections

6. Final Validation:
   - Verify against common error patterns
   - Check for String::from_utf8 on ABI data
   - Ensure all trait imports are present
   - Confirm export macro is correct
   - Validate all data structures derive Clone
   - Verify proper error handling throughout

7. Output Component:
   - Present Cargo.toml and lib.rs content
   - Provide clear building and testing instructions
   - Explain component behavior and usage

## Component Knowledge Base

Reference the templates and patterns in WAVSai-claude.md for different component types:
- ENS resolvers
- Token balance checkers
- Price oracles
- Gas price estimators
- API integrators

## Critical Error Prevention

Always check for these critical errors:
- Using String::from_utf8 on ABI data
- Missing trait imports (e.g., Provider trait)
- Type conversion issues with blockchain types
- Using map_err on Option types
- Incorrect export macro usage
- Memory management issues

## Output Format

Always provide the component in this format:
1. Understanding confirmation
2. Component files (Cargo.toml, lib.rs)
3. Building and testing instructions
4. Expected behavior description
```

## Implementation Steps

1. Create a Claude Code plugin or configuration file that:
   - Detects WAVS component requests
   - Activates the WAVSai system prompt
   - Guides Claude through the WAVSai process

2. Test the integration with various component requests:
   - Simple components (e.g., "Hello World")
   - API-based components (e.g., weather data)
   - Blockchain components (e.g., token balance)

3. Refine the detection and process based on testing results:
   - Improve detection accuracy
   - Enhance process guidance
   - Expand component knowledge

4. Deploy the integration to Claude Code:
   - Enable for all users
   - Monitor success rates
   - Gather feedback for improvements

## Success Metrics

Monitor these metrics to evaluate the integration:
- Detection accuracy (% of component requests correctly identified)
- Process adherence (% of steps followed correctly)
- Component success rate (% of components that work on first try)
- User satisfaction (ratings or feedback)
- Development time reduction (compared to manual component creation)

## Caveats and Limitations

- Claude may need additional context for very specialized components
- Some complex APIs may require user-provided documentation
- Certain edge cases in blockchain interactions may need special handling
- The system will need regular updates as WAVS evolves